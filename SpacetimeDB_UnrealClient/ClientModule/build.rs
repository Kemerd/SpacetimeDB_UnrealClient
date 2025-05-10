fn main() {
    // Use our own header directory that doesn't depend on Unreal Engine headers
    let cpp_src_dir = "cpp_src";
    
    // Build the C++ bridge code
    let mut build = cxx_build::bridge("src/ffi.rs");
    
    // Add C++ standard based on platform
    if cfg!(target_os = "windows") {
        // For MSVC compiler on Windows
        build.flag("/std:c++17");
    } else {
        // For other platforms
        build.flag_if_supported("-std=c++17");
    }
    
    // Add our own include directory with simplified headers
    build.include(cpp_src_dir)
        // Add include directories if needed
        // .include("cpp/include")
        // Compile C++ files if any
        // .file("cpp/source/example.cpp")
        .compile("stdb_client_bridge");
    
    // Create target/cxxbridge directory if it doesn't exist
    let target_dir = std::path::Path::new("target/cxxbridge");
    if !target_dir.exists() {
        std::fs::create_dir_all(target_dir).expect("Failed to create target/cxxbridge directory");
    }
    
    // Copy bridge.h from cpp_src to target/cxxbridge
    let bridge_h_src = std::path::Path::new(cpp_src_dir).join("bridge.h");
    let bridge_h_dst = target_dir.join("bridge.h");
    std::fs::copy(&bridge_h_src, &bridge_h_dst).expect("Failed to copy bridge.h");
    
    println!("cargo:warning=Copied {} to {}", bridge_h_src.display(), bridge_h_dst.display());
    
    // Tell cargo to invalidate the built crate when the wrapper changes
    println!("cargo:rerun-if-changed=src/ffi.rs");
    
    // Tell cargo to invalidate the built crate when any C++ source files change
    println!("cargo:rerun-if-changed=cpp");
    
    // Tell cargo to invalidate the built crate when our headers change
    println!("cargo:rerun-if-changed={}", cpp_src_dir);
    
    // Add link directives for external libraries if needed
    // println!("cargo:rustc-link-lib=dylib=example");
} 