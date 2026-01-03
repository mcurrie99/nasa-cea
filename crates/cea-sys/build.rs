use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=CEA_DIR");
    println!("cargo:rerun-if-env-changed=CEA_INCLUDE_DIR");
    println!("cargo:rerun-if-env-changed=CEA_LIB_DIR");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // -------- Locate include/lib --------
    let (include_dir, lib_dir) = if env::var_os("CARGO_FEATURE_VENDORED").is_some() {
        // Assumes you have CEA checked out at vendor/cea
        let cea_src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../vendor/cea");

        // Configure minimal build: Fortran + C binding, no Python.
        // README mentions a Fortran+C preset and flags to disable Python :contentReference[oaicite:4]{index=4}.
        let dst = cmake::Config::new(&cea_src)
            .define("CEA_ENABLE_BIND_C", "ON")
            .define("CEA_ENABLE_BIND_PYTHON", "OFF")
            .define("CEA_ENABLE_BIND_MATLAB", "OFF")
            .define("CEA_BUILD_TESTING", "OFF")
            .build();

        (dst.join("include"), dst.join("lib"))
    } else {
        // System install: prefer explicit env vars
        if let Ok(prefix) = env::var("CEA_DIR") {
            let p = PathBuf::from(prefix);
            (p.join("include"), p.join("lib"))
        } else {
            let inc = env::var("CEA_INCLUDE_DIR").expect("Set CEA_DIR or CEA_INCLUDE_DIR/CEA_LIB_DIR");
            let lib = env::var("CEA_LIB_DIR").expect("Set CEA_DIR or CEA_INCLUDE_DIR/CEA_LIB_DIR");
            (PathBuf::from(inc), PathBuf::from(lib))
        }
    };

    // Tell Rust where the library is and what to link
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=cea"); // typically links libcea.*

    // -------- Bindgen --------
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", include_dir.display()))
        .allowlist_function("^cea_.*")
        .allowlist_type("^cea_.*")
        .allowlist_var("^CEA_.*")
        .generate()
        .expect("bindgen failed");

    let bindings_path = out_dir.join("bindings.rs");
    bindings
        .write_to_file(&bindings_path)
        .expect("could not write bindings");
}
