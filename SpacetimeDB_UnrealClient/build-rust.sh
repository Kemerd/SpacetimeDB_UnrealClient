#!/bin/bash
# Build script for SpacetimeDB Unreal Client Rust components
# This builds the ClientModule for use with Unreal Engine on Unix-like systems (macOS, Linux)

set -e

echo "Building SpacetimeDB Unreal Client Rust components..."

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
echo "Script directory: $SCRIPT_DIR"

# Verify cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "Rust and Cargo must be installed to build the SpacetimeDB Unreal Client."
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi

# Verify cxxbridge is installed
if ! command -v cxxbridge &> /dev/null; then
    echo "cxxbridge command not found. Installing cxxbridge-cmd..."
    cargo install cxxbridge-cmd
    if [ $? -ne 0 ]; then
        echo "Failed to install cxxbridge-cmd."
        exit 1
    fi
fi

# Set build configuration based on UE_BUILD_CONFIGURATION env var if present
if [ "$UE_BUILD_CONFIGURATION" == "Debug" ]; then
    CARGO_PROFILE="debug"
elif [ "$UE_BUILD_CONFIGURATION" == "Development" ]; then
    CARGO_PROFILE="release"
elif [ "$UE_BUILD_CONFIGURATION" == "Shipping" ]; then
    CARGO_PROFILE="release"
else
    CARGO_PROFILE="release"
fi

echo "Building with profile: $CARGO_PROFILE"

# Change to ClientModule directory
cd "$SCRIPT_DIR/ClientModule"

# Create directory for CXX bridge headers if it doesn't exist
mkdir -p "target/cxxbridge"

# Generate CXX bridge headers
echo "Generating CXX bridge headers..."
cxxbridge src/ffi.rs --header > target/cxxbridge/ffi.h
if [ $? -ne 0 ]; then
    echo "Failed to generate CXX bridge headers."
    exit 1
fi

# Build the rust library
echo "Running cargo build with profile $CARGO_PROFILE..."
if [ "$CARGO_PROFILE" == "debug" ]; then
    cargo build
else
    cargo build --release
fi

if [ $? -ne 0 ]; then
    echo "Failed to build Rust library."
    exit 1
fi

echo "Build completed successfully!"
exit 0 