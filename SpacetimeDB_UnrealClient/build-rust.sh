#!/bin/bash
# Build script for SpacetimeDB Unreal Client Rust components
# This builds the ClientModule for use with Unreal Engine on macOS/Linux

set -e

echo "Building SpacetimeDB Unreal Client Rust components..."

# Get the script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
echo "Script directory: $SCRIPT_DIR"

# Verify cargo is installed
if ! command -v cargo &> /dev/null
then
    echo "Rust and Cargo must be installed to build the SpacetimeDB Unreal Client."
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi

# Set build configuration based on UE_BUILD_CONFIGURATION env var if present
if [ "$UE_BUILD_CONFIGURATION" = "Debug" ]; then
    CARGO_PROFILE="debug"
elif [ "$UE_BUILD_CONFIGURATION" = "Development" ]; then
    CARGO_PROFILE="release"
elif [ "$UE_BUILD_CONFIGURATION" = "Shipping" ]; then
    CARGO_PROFILE="release"
else
    CARGO_PROFILE="release"
fi

echo "Building with profile: $CARGO_PROFILE"

# Change to ClientModule directory
cd "$SCRIPT_DIR/ClientModule"

# Build the rust library
echo "Running cargo build with profile $CARGO_PROFILE..."
if [ "$CARGO_PROFILE" = "debug" ]; then
    cargo build
else
    cargo build --release
fi

echo "Build completed successfully!"
exit 0 