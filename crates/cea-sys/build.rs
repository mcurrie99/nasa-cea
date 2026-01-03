use std::{env, path::{Path, PathBuf}};
use walkdir::WalkDir;

fn find_named_file(root: &Path, filename: &str) -> Option<PathBuf> {
    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() && entry.file_name() == filename {
            return Some(entry.into_path());
        }
    }
    None
}

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-env-changed=CEA_SYS_CMAKE_GENERATOR");
    println!("cargo:rerun-if-env-changed=FC");
    println!("cargo:rerun-if-env-changed=CC");
    println!("cargo:rerun-if-env-changed=CXX");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let cea_src = manifest_dir.join("vendor").join("cea");

    // Build/install into a stable location under OUT_DIR
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let install_prefix = out_dir.join("cea-install");

    let mut cfg = cmake::Config::new(&cea_src);

    // Match CEA docs: minimal "Fortran + C" build disables Python binding
    // (equivalent intent to their "core-c" preset) :contentReference[oaicite:5]{index=5}
    cfg.define("CEA_ENABLE_BIND_PYTHON", "OFF");
    cfg.define("CEA_BUILD_TESTING", "OFF");

    // Keep shared by default (simpler with Fortran runtime); you can experiment later:
    // cfg.define("BUILD_SHARED_LIBS", "OFF");

    if let Ok(generator) = env::var("CEA_SYS_CMAKE_GENERATOR") {
        cfg.generator(generator);
    }

    // cmake crate installs to OUT_DIR by default; we’ll force an explicit install prefix
    cfg.define("CMAKE_INSTALL_PREFIX", &install_prefix);

    // Build + install
    let dst = cfg.build();

    // Link search path
    let libdir_candidates = [dst.join("lib"), dst.join("lib64")];
    let libdir = libdir_candidates
        .into_iter()
        .find(|p| p.exists())
        .unwrap_or(dst.join("lib"));

    println!("cargo:rustc-link-search=native={}", libdir.display());
    println!("cargo:rustc-link-lib=cea"); // links libcea* :contentReference[oaicite:6]{index=6}

    // On mac/linux, embed rpath so binaries can find libcea without env vars.
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "linux" || target_os == "macos" {
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", libdir.display());
    }

    // Bindgen against installed header. C API is via <cea.h>. :contentReference[oaicite:7]{index=7}
    let include_dir = dst.join("include");
    let header = include_dir.join("cea.h");

    let bindings = bindgen::Builder::default()
        .header(header.to_string_lossy())
        .clang_arg(format!("-I{}", include_dir.display()))
        .allowlist_function("^cea_.*")
        .allowlist_type("^cea_.*")
        .allowlist_var("^CEA_.*")
        .derive_default(true)
        .generate_comments(true)
        .generate()
        .expect("bindgen failed");

    let bindings_out = out_dir.join("bindings.rs");
    bindings
        .write_to_file(&bindings_out)
        .expect("could not write bindings.rs");

    // Expose install prefix + best-effort database file paths (CEA installs default databases) :contentReference[oaicite:8]{index=8}
    println!("cargo:rustc-env=CEA_SYS_PREFIX={}", dst.display());

    if let Some(p) = find_named_file(&dst, "thermo.lib") {
        println!("cargo:rustc-env=CEA_SYS_THERMO_LIB={}", p.display());
    }
    if let Some(p) = find_named_file(&dst, "trans.lib") {
        println!("cargo:rustc-env=CEA_SYS_TRANS_LIB={}", p.display());
    }
}
