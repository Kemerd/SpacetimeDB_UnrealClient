# SpacetimeDB Unreal Client Plugin

This plugin provides integration between Unreal Engine and SpacetimeDB, allowing games to connect to SpacetimeDB databases for multiplayer functionality and cloud-based game state.

## Features
- **Custom NetDriver**: Seamless integration of SpacetimeDB with Unreal's replication system
- **GameInstanceSubsystem**: Manages database connections, subscriptions, and data synchronization
- **Cross-platform Support**: Works on Windows, macOS, and Linux
- **Automated Build Process**: Automatically builds the Rust code during plugin compilation
- Connect to SpacetimeDB instances
- Call SpacetimeDB reducers
- Subscribe to SpacetimeDB tables
- Handle events and state updates

## Architecture

The plugin uses a layered architecture:

1. **Rust Core**: The low-level client implemented in Rust using the SpacetimeDB SDK
2. **C++ FFI Layer**: Generated C++ bindings that bridge between Rust and C++
3. **Unreal-specific C++ Layer**: UE-friendly wrappers and classes
4. **Blueprint Integration**: Blueprint nodes for designers to use without C++ coding

## Prerequisites
- Unreal Engine 5.5
- Rust (1.70+) - Install from [https://rustup.rs/](https://rustup.rs/)
- cxxbridge (`cargo install cxxbridge-cmd`)
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
└── SpacetimeDB_UnrealClient.uplugin  # Plugin definition
```

## Build Process

The plugin uses a custom build process to compile the Rust code and integrate it with Unreal Engine:

1. The `SpacetimeDB_UnrealClient.Build.cs` file detects the platform and runs the appropriate build script.
2. `build-rust-win.bat` (Windows) or `build-rust.sh` (Linux/Mac) builds the Rust library and generates C++ bindings.
3. The Rust `build.rs` script uses `cxx-build` to generate the necessary interop code.
4. Unreal Engine compiles the C++ code and links it with the Rust library.

### Cross-Compilation Details

The plugin uses [CXX](https://github.com/dtolnay/cxx) for seamless Rust-C++ interoperability:

1. The Rust `lib.rs` file defines a `cxx::bridge` module that specifies the FFI interface.
2. During build, CXX generates matching C++ headers and implementation.
3. The generated C++ code safely marshals data between Rust and C++.

### CI/CD Integration

The build system supports CI/CD scenarios with the following environment variables:

- `SPACETIMEDB_SKIP_RUST_BUILD`: Set to "1" to skip the Rust build step (useful if libraries are pre-built)
- `SPACETIMEDB_RUST_LIB_PATH`: Path to pre-built Rust libraries for CI/CD environments

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

1. Add the plugin to your Unreal Engine project
2. Use the `USpacetimeDBSubsystem` to connect to a SpacetimeDB instance:

```cpp
USpacetimeDBSubsystem* SpacetimeDBSubsystem = GetGameInstance()->GetSubsystem<USpacetimeDBSubsystem>();
SpacetimeDBSubsystem->Connect("localhost:3000", "my_database");
```

3. Subscribe to tables:

```cpp
TArray<FString> Tables;
Tables.Add("players");
Tables.Add("game_state");
SpacetimeDBSubsystem->SubscribeToTables(Tables);
```

4. Call reducers:

```cpp
SpacetimeDBSubsystem->CallReducer("move_player", "{\"position\": {\"x\": 100, \"y\": 200}}");
```

5. Handle events using delegates:

```cpp
SpacetimeDBSubsystem->OnEventReceived.AddDynamic(this, &AMyGameMode::HandleTableEvent);
```

## Configuration
The following settings can be configured in your project's settings:
- `SPACETIME_HOST`: The SpacetimeDB server address
- `SPACETIME_DBNAME`: The database name
- `SPACETIME_AUTH_TOKEN`: Authentication token (if required)

## Troubleshooting

If you encounter build issues:

1. Make sure Rust and cxxbridge are installed
2. Check the Rust build logs in the `rust/target` directory
3. Verify the generated C++ bindings at `rust/stdb.hpp`
4. Check that the library is being built in the correct configuration (debug/release)

## License
MIT License
