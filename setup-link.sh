#!/bin/bash
# setup-link.sh
# Script to create a symbolic link from an Unreal Engine project to the SpacetimeDB_UnrealClient plugin
# This allows for easier development and testing of the plugin

echo "===================================================="
echo "SpacetimeDB Unreal Client Plugin - Link Setup Script"
echo "===================================================="
echo ""

# Get the absolute path of the plugin directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PLUGIN_PATH="${SCRIPT_DIR}/SpacetimeDB_UnrealClient"
PROJECT_PATH="$1"

# Check if a project path was provided
if [ -z "$PROJECT_PATH" ]; then
    echo "ERROR: Missing project path parameter."
    echo ""
    echo "Usage: ./setup-link.sh /path/to/UE_Project"
    echo "Example: ./setup-link.sh ~/Projects/MyUnrealProject"
    echo ""
    exit 1
fi

# Convert to absolute path if it's not already
if [[ "$PROJECT_PATH" != /* ]]; then
    PROJECT_PATH="$(cd "$(dirname "$PROJECT_PATH")" && pwd)/$(basename "$PROJECT_PATH")"
fi

echo "Project path: $PROJECT_PATH"
echo "Plugin path: $PLUGIN_PATH"
echo ""

# Check if directories exist
if [ ! -d "$PLUGIN_PATH" ]; then
    echo "ERROR: Plugin directory not found at \"$PLUGIN_PATH\""
    echo ""
    exit 1
fi

if [ ! -d "$PROJECT_PATH" ]; then
    echo "ERROR: Project directory not found at \"$PROJECT_PATH\""
    echo ""
    exit 1
fi

# Create Plugins directory if it doesn't exist
if [ ! -d "$PROJECT_PATH/Plugins" ]; then
    echo "Creating Plugins directory..."
    mkdir -p "$PROJECT_PATH/Plugins"
fi

# Create symbolic link
echo "Creating symbolic link..."
TARGET_PATH="$PROJECT_PATH/Plugins/SpacetimeDB_UnrealClient"

# Remove existing link/directory if it exists
if [ -e "$TARGET_PATH" ]; then
    echo "Removing existing link or directory..."
    rm -rf "$TARGET_PATH"
    
    if [ -e "$TARGET_PATH" ]; then
        echo ""
        echo "ERROR: Failed to remove existing directory."
        echo "Please remove it manually: $TARGET_PATH"
        echo ""
        exit 1
    fi
fi

# Create the symbolic link
ln -s "$PLUGIN_PATH" "$TARGET_PATH"

if [ $? -ne 0 ]; then
    echo ""
    echo "ERROR: Failed to create symbolic link."
    echo "Make sure you have the right permissions."
    echo ""
    echo "Alternatively, you can manually copy the plugin to your project using:"
    echo "cp -R \"$PLUGIN_PATH\" \"$TARGET_PATH\""
    echo ""
    exit 1
fi

echo ""
echo "===================================================="
echo "Success! SpacetimeDB Unreal Client plugin linked."
echo "===================================================="
echo ""
echo "Plugin linked to: $PROJECT_PATH/Plugins/"
echo ""
echo "IMPORTANT: You need to regenerate your project files" 
echo "and rebuild your project in your IDE."
echo ""
echo "====================================================" 
echo ""

exit 0 