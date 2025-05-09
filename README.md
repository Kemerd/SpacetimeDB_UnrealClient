# SpacetimeDB Unreal Client Plugin

## Overview
This plugin integrates SpacetimeDB with Unreal Engine 5.5 by providing a C++ interface to the SpacetimeDB client SDK. It handles the FFI (Foreign Function Interface) between C++ and Rust using the `cxx` crate for seamless integration.

## Features
- **Custom NetDriver**: Seamless integration of SpacetimeDB with Unreal's replication system
- **GameInstanceSubsystem**: Manages database connections, subscriptions, and data synchronization
- **Cross-platform Support**: Works on Windows, macOS, and Linux
- **Automated Build Process**: Automatically builds the Rust code during plugin compilation

## Prerequisites
- Unreal Engine 5.5
- Rust (1.70+) - Install from [https://rustup.rs/](https://rustup.rs/)
- A C++ compiler compatible with your platform
  - Windows: Visual Studio 2022
  - macOS: Xcode
  - Linux: GCC or Clang
- SpacetimeDB CLI - Optional but recommended

## Project Structure
```
/SpacetimeDB_UnrealClient/
├── /Source/
│   └── /SpacetimeDB_UnrealClient/
│       ├── SpacetimeDB_UnrealClient.Build.cs  # UE build script
│       ├── SpacetimeDB_UnrealClient.h         # Module header
│       └── SpacetimeDB_UnrealClient.cpp       # Module implementation
├── /rust/
│   ├── Cargo.toml        # Rust package definition
│   ├── src/lib.rs        # Rust library with FFI definitions
│   ├── build.rs          # Rust build script for C++ binding generation
│   ├── build-rust-win.bat # Windows build script
│   └── build-rust.sh     # Unix build script
├── SpacetimeDB_UnrealClient.uplugin  # Plugin definition
└── setup-link.bat        # Windows script to create symbolic links
```

## Installation

### Windows
1. Clone this repository:
```
git clone <repo-url>
cd SpacetimeDB_UnrealClient
```

2. Ensure Rust is installed:
```
rustup update stable
```

3. Link the plugin to your Unreal Engine project using the provided script:
```
setup-link.bat C:\Path\To\Your\UE_Project
```

4. Regenerate your project files and build your project in Visual Studio

### macOS/Linux
1. Clone this repository:
```
git clone <repo-url>
cd SpacetimeDB_UnrealClient
```

2. Ensure Rust is installed:
```
rustup update stable
```

3. Create a symbolic link to your Unreal Engine project:
```
ln -s "$(pwd)/SpacetimeDB_UnrealClient" "/path/to/your/UE_Project/Plugins/SpacetimeDB_UnrealClient"
```

4. Regenerate your project files and build your project

## Usage
1. Enable the plugin in your Unreal Engine project
2. Configure your SpacetimeDB connection in the project settings
3. Use the SpacetimeDB functionality in your Blueprints or C++ code

## Configuration
The following settings can be configured in your project's settings:
- `SPACETIME_HOST`: The SpacetimeDB server address
- `SPACETIME_DBNAME`: The database name
- `SPACETIME_AUTH_TOKEN`: Authentication token (if required)

## License
MIT License
