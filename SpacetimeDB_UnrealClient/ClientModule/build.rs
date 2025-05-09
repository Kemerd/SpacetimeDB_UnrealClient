fn main() {
    // Build the C++ bridge code
    cxx_build::bridge("src/ffi.rs")
        // Add C++ standard
        .flag_if_supported("-std=c++17")
        // Add include directories if needed
        // .include("cpp/include")
        // Compile C++ files if any
        // .file("cpp/source/example.cpp")
        .compile("stdb_client_bridge");
    
    // Tell cargo to invalidate the built crate when the wrapper changes
    println!("cargo:rerun-if-changed=src/ffi.rs");
    
    // Tell cargo to invalidate the built crate when any C++ source files change
    println!("cargo:rerun-if-changed=cpp");
    
    // Add link directives for external libraries if needed
    // println!("cargo:rustc-link-lib=dylib=example");
} 