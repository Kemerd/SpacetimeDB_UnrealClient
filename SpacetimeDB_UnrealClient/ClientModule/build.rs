fn main() {
    // Use our own header directory that doesn't depend on Unreal Engine headers
    let cpp_src_dir = "cpp_src";
    
    // Build the C++ bridge code
    cxx_build::bridge("src/ffi.rs")
        // Add C++ standard
        .flag_if_supported("-std=c++17")
        // Add our own include directory with simplified headers
        .include(cpp_src_dir)
        // Add include directories if needed
        // .include("cpp/include")
        // Compile C++ files if any
        // .file("cpp/source/example.cpp")
        .compile("stdb_client_bridge");
    
    // Tell cargo to invalidate the built crate when the wrapper changes
    println!("cargo:rerun-if-changed=src/ffi.rs");
    
    // Tell cargo to invalidate the built crate when any C++ source files change
    println!("cargo:rerun-if-changed=cpp");
    
    // Tell cargo to invalidate the built crate when our headers change
    println!("cargo:rerun-if-changed={}", cpp_src_dir);
    
    // Add link directives for external libraries if needed
    // println!("cargo:rustc-link-lib=dylib=example");
} 