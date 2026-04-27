// build.rs - Multi-compiler build (MSVC, MinGW, GCC)
use std::env;

fn main() {
    println!("cargo:rerun-if-changed=cpp/");

    // Debug info
    let target = env::var("TARGET").unwrap_or_default();
    println!("cargo:warning=Building for target: {target}");

    let mut build = cc::Build::new();
    build.cpp(true).file("cpp/lib.cpp").include("cpp");

    // Configure based on target
    if target.contains("msvc") {
        // MSVC-specific flags
        build
            .flag("/std:c++17")
            .flag("/O2") // Optimization level 2
            .flag("/EHsc") // Exception handling model
            .define("_CRT_SECURE_NO_WARNINGS", None); // Disable security warnings
    } else if target.contains("gnu") || target.contains("mingw") {
        // MinGW/GCC flags for Windows GNU target
        build
            .flag("-O3")
            .flag("-std=c++17")
            .flag("-Wall")
            .flag("-Wextra")
            .flag("-static-libgcc")
            .flag("-static-libstdc++");

        // Add Windows-specific flags for MinGW
        if target.contains("windows") {
            build.define("WIN32", None);
            build.define("_WIN32", None);
        }
    } else {
        // Default GCC flags for other targets
        build
            .flag("-O3")
            .flag("-std=c++17")
            .flag("-Wall")
            .flag("-Wextra")
            .flag("-pedantic");
    }

    build.compile("optimal5");

    println!("cargo:rustc-link-lib=static=optimal5");
}
