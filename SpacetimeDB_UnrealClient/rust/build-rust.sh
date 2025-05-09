#!/bin/bash
set -e

echo "Building SpacetimeDB Rust client..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Rust is not installed. Please install Rust from https://rustup.rs/"
    exit 1
fi

# Check if cxxbridge is installed
if ! command -v cxxbridge &> /dev/null; then
    echo "cxxbridge is not installed. Installing..."
    cargo install cxxbridge-cmd
    if [ $? -ne 0 ]; then
        echo "Failed to install cxxbridge. Please install manually: cargo install cxxbridge-cmd"
        exit 1
    fi
fi

# Navigate to the Rust directory
cd "$(dirname "$0")"

# Build the Rust library
cargo build --release
if [ $? -ne 0 ]; then
    echo "Failed to build Rust library"
    exit 1
fi

# Generate C++ headers
cxxbridge --header > stdb.hpp
if [ $? -ne 0 ]; then
    echo "Failed to generate C++ headers"
    exit 1
fi

echo "SpacetimeDB Rust client built successfully!"
exit 0 