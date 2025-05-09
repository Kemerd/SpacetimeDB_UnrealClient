#!/bin/bash
# setup-spacetimedb.sh
# Script to automate SpacetimeDB setup for Unix/Linux/macOS
# This script installs all the necessary components for SpacetimeDB development

echo "===================================================="
echo "SpacetimeDB Setup Script for Unix/Linux/macOS"
echo "===================================================="
echo ""

# Check if Rust is installed
if ! command -v rustc &> /dev/null; then
    echo "Rust is not installed. Installing Rust..."
    echo "This will download and run the rustup installer."
    echo ""
    
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    
    echo ""
    echo "Adding Rust to the current PATH..."
    source "$HOME/.cargo/env"
else
    echo "Rust is already installed. Updating..."
    rustup update stable
fi

echo ""
echo "Installing cxxbridge command..."
cargo install cxxbridge-cmd

echo ""
echo "Checking for SpacetimeDB CLI..."
if ! command -v spacetime &> /dev/null; then
    echo "SpacetimeDB CLI is not installed. Installing..."
    
    # Determine OS type
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        echo "Detected macOS. Installing with Homebrew..."
        if ! command -v brew &> /dev/null; then
            echo "Homebrew not found. Please install Homebrew first: https://brew.sh/"
            echo "Then run this script again."
            exit 1
        fi
        
        brew install clockworklabs/tap/spacetime
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        # Linux
        echo "Detected Linux. Installing with curl..."
        curl -fsSL https://docs.spacetimedb.com/install.sh | bash
    else
        echo "Unsupported operating system. Please visit https://spacetimedb.com/install for manual installation instructions."
        exit 1
    fi
else
    echo "SpacetimeDB CLI is already installed."
fi

echo ""
echo "Starting local SpacetimeDB instance..."
# Start in a new terminal window based on the OS
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    osascript -e 'tell app "Terminal" to do script "spacetime start"'
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Linux - try common terminal emulators
    if command -v gnome-terminal &> /dev/null; then
        gnome-terminal -- bash -c "spacetime start; exec bash"
    elif command -v xterm &> /dev/null; then
        xterm -e "spacetime start; exec bash" &
    else
        echo "Could not launch a new terminal window automatically."
        echo "Please open a new terminal and run: spacetime start"
    fi
fi

echo ""
echo "===================================================="
echo "SpacetimeDB setup is complete"
echo "===================================================="
echo ""
echo "For local development:"
echo " - In a terminal window, run: spacetime start"
echo " - In another terminal, run: spacetime login"
echo ""
echo "To create a new project:"
echo " - Run: spacetime init --lang rust"
echo " - or: spacetime init --lang csharp"
echo ""
echo "Docs: https://spacetimedb.com/docs"
echo ""
echo "====================================================" 