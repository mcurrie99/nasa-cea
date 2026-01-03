use std::{
    env,
    ffi::OsStr,
    path::{Path, PathBuf},
};

use walkdir::WalkDir;

fn main() {
    // Rebuild triggers
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-env-changed=CEA_SYS_CMAKE_GENERATOR");
    println!("cargo:rerun-if-env-changed=FC");
    println!("cargo:rerun-if-env-changed=CC");
    println!("cargo:rerun-if-env-changed=CXX");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let cea_src = manifest_dir.join("vendor").join("cea");
    if !cea_src.exists() {
        panic!(
            "Vendored CEA source not found at {} (expected crates/cea-sys/vendor/cea)",
            cea_src.display()
        );
    }

    // Export paths for the Rust crate (compile-time via cargo:rustc-env)
    let include_dir = cea_src.join("source").join("bind").join("c");
    println!("cargo:rustc-env=CEA_SYS_VENDOR_DIR={}", cea_src.display());
    println!(
        "cargo:rustc-env=CEA_SYS_INCLUDE_DIR={}",
        include_dir.display()
    );

    // Optional: export database paths if present
    let thermo_lib = cea_src.join("data").join("thermo.lib");
    let trans_lib = cea_src.join("data").join("trans.lib");
    if thermo_lib.exists() {
        println!("cargo:rustc-env=CEA_SYS_THERMO_LIB={}", thermo_lib.display());
    }
    if trans_lib.exists() {
        println!("cargo:rustc-env=CEA_SYS_TRANS_LIB={}", trans_lib.display());
    }

    // -------------------------
    // Build vendored CEA (C API)
    // -------------------------
    let mut cfg = cmake::Config::new(&cea_src);

    // Minimal build: disable wrapper stacks that drag Python/Cython in.
    cfg.define("CEA_BUILD_TESTING", "OFF")
        .define("CEA_ENABLE_BIND_PYTHON", "OFF")
        .define("CEA_ENABLE_BIND_MATLAB", "OFF")
        .define("CEA_ENABLE_BIND_EXCEL", "OFF");

    // Critical: build only the C binding target (don’t run "install")
    cfg.build_target("cea_bindc");

    if let Ok(generator) = env::var("CEA_SYS_CMAKE_GENERATOR") {
        cfg.generator(generator);
    }

    let dst = cfg.build();
    let build_root = dst.join("build");

    // -------------------------
    // Link libraries
    // -------------------------
    // Static libs require order: dependent first, dependencies after.
    // We link all 3 to be safe across platforms/build modes.
    link_target(&build_root, "cea_bindc");
    link_target(&build_root, "cea_core");
    link_target(&build_root, "fbasics_core");

    // -------------------------
    // Bindgen
    // -------------------------
    let wrapper_h = manifest_dir.join("wrapper.h");
    if !wrapper_h.exists() {
        panic!("wrapper.h not found at {}", wrapper_h.display());
    }

    let bindings = bindgen::Builder::default()
        .header(wrapper_h.to_string_lossy())
        .clang_arg(format!("-I{}", include_dir.display()))
        .allowlist_function("^cea_.*")
        .allowlist_type("^cea_.*")
        .allowlist_var("^CEA_.*")
        .opaque_type("cea_.*_t")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("bindgen failed");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("could not write bindings.rs");
}

fn link_target(build_root: &Path, name: &str) {
    // Prefer shared, then static, then windows import libs
    let candidates = [
        format!("lib{name}.dylib"),
        format!("lib{name}.so"),
        format!("lib{name}.a"),
        format!("{name}.lib"),
        format!("lib{name}.lib"),
    ];

    let mut found: Option<PathBuf> = None;
    'outer: for entry in WalkDir::new(build_root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let file = entry.file_name().to_string_lossy();
        for c in &candidates {
            if file == *c {
                found = Some(entry.path().to_path_buf());
                break 'outer;
            }
        }
    }

    let lib_path = found.unwrap_or_else(|| {
        panic!(
            "Could not find built library for target '{name}' under {}",
            build_root.display()
        )
    });

    let lib_dir = lib_path.parent().unwrap();
    println!("cargo:rustc-link-search=native={}", lib_dir.display());

    let ext = lib_path.extension().and_then(OsStr::to_str).unwrap_or("");
    let is_static = ext == "a";

    if is_static {
        println!("cargo:rustc-link-lib=static={name}");
    } else {
        println!("cargo:rustc-link-lib={name}");

        // Help runtime loading for dylibs in nonstandard locations
        let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
        if target_os == "macos" || target_os == "linux" {
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir.display());
        }
    }
}
