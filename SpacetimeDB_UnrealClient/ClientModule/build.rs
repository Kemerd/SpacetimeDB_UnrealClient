fn main() {
    // Use our own header directory that doesn't depend on Unreal Engine headers
    let cpp_src_dir = "cpp_src";
    
    // Build the C++ bridge code
    let mut build = cxx_build::bridge("src/ffi.rs");
    
    // The CXXFLAGS (like /std:c++20, /MD) are now primarily set in the build-rust-win.bat script
    // to ensure they apply globally to the cargo build process, including dependencies like the cxx crate.
    // We still add /MD here as a fallback or for non-Windows if needed, though the batch script is key for Windows.
    if cfg!(target_os = "windows") {
        build.flag("/MD"); // Ensure dynamic runtime for MSVC
        build.flag("/std:c++20"); // Explicitly set C++20 standard
        build.flag("/Zc:__cplusplus"); // Ensure __cplusplus macro is correct for C++20
        build.flag("/permissive-"); // Enforce C++ standards conformance
                           // Other flags like /std:c++20, /Zc:__cplusplus, /permissive- are set by CXXFLAGS in .bat

        // Explicitly link the dynamic C++ standard library. This can help when /MD alone isn't enough
        // to resolve specific STL symbols during the final link stage by Unreal Engine.
        println!("cargo:rustc-link-lib=msvcprt");
    } else {
        // For other platforms, ensure C++20 if supported
        build.flag_if_supported("-std=c++20");
    }
    
    // Add our own include directory with simplified headers
    build.include(cpp_src_dir)
        // Add include directories if needed
        // .include("cpp/include")
        // Compile C++ files if any
        // .file("cpp/source/example.cpp")
        .compile("stdb_client_bridge");
    
    // Create target/cxxbridge directory if it doesn't exist
    let target_dir_path = std::path::Path::new("target/x86_64-pc-windows-msvc/cxxbridge");
    if !target_dir_path.exists() {
        std::fs::create_dir_all(target_dir_path).expect("Failed to create target/cxxbridge directory");
    }

    // The cxx_build::bridge call generates ffi.h in target/cxxbridge directory
    // We only need to copy the handwritten headers from cpp_src
    let headers_to_copy = vec!["bridge.h", "UnrealReplication.h"];

    for header_name in headers_to_copy {
        let src_path = std::path::Path::new(cpp_src_dir).join(header_name);
        let dst_path = target_dir_path.join(header_name);

        if src_path.exists() {
            std::fs::copy(&src_path, &dst_path)
                .expect(&format!("Failed to copy {}", header_name));
            println!(
                "cargo:warning=Copied {} to {}",
                src_path.display(),
                dst_path.display()
            );
        } else {
            println!(
                "cargo:warning=Source {} not found at {}",
                header_name,
                src_path.display()
            );
        }
    }
    
    // Copy the generated ffi.h from target/cxxbridge to our target directory
    // This is only needed if the paths are different
    let generated_cxxbridge_dir = std::path::Path::new("target/cxxbridge");
    let ffi_src_path = generated_cxxbridge_dir.join("ffi.h");
    if generated_cxxbridge_dir.exists() && ffi_src_path.exists() {
        let ffi_dst_path = target_dir_path.join("ffi.h");
        std::fs::copy(&ffi_src_path, &ffi_dst_path)
            .expect("Failed to copy ffi.h");
        println!(
            "cargo:warning=Copied {} to {}",
            ffi_src_path.display(),
            ffi_dst_path.display()
        );
    } else {
        println!(
            "cargo:warning=Generated ffi.h not found at {}",
            ffi_src_path.display()
        );
    }
    
    println!("cargo:rerun-if-changed=src/ffi.rs");
    println!("cargo:rerun-if-changed=cpp_src"); // Watch the whole cpp_src directory
    
    // Add explicit linkage options for C++ runtime compatibility with Unreal
    // The /MD flag passed to the cxx_build compiler should handle instructing
    // the linker to use the correct dynamic runtime libraries. We will rely on that
    // and the environment setup by build-rust-win.bat for the linker to find them.
    // if cfg!(target_os = "windows") {
        // Link against the dynamic C runtime library that matches Unreal Engine
        // println!("cargo:rustc-link-lib=msvcrt");
        // println!("cargo:rustc-link-lib=vcruntime");
        // println!("cargo:rustc-link-lib=ucrt");
        
        // Add C++ standard library - specifically needed for __std_mismatch_1 and other C++ symbols
        // Use the dynamic version to match Unreal Engine and our /MD flag
        // println!("cargo:rustc-link-lib=msvcprt");  // Dynamic C++ standard library
        
        // Note: We no longer hardcode paths here - rely on environment variables
        // set in the build-rust-win.bat script instead for consistent toolchain usage
    // }
}
