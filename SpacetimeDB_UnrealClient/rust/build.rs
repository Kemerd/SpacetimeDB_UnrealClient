// build.rs
fn main() {
    cxx_build::bridge("src/lib.rs")
        .flag_if_supported("-std=c++14")
        .compile("spacetimedb_client");
    
    println!("cargo:rerun-if-changed=src/lib.rs");
} 