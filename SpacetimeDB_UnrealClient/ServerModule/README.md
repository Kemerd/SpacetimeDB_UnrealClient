# ServerModule

This module provides the server-side implementation of the SpacetimeDB Unreal Client Plugin. It handles actor spawning, destruction, property replication, RPCs, and network relevancy on the server.

## Purpose

The ServerModule serves as the authoritative game state manager, handling:
1. UObject and Actor lifecycle management
2. Property replication and synchronization
3. Remote procedure calls (RPCs)
4. Client connection management
5. Network relevancy and interest management

## Structure

- `src/lib.rs` - Module entry point with initialization and core functionality
- `src/object/` - Core UObject functionality (base for all Unreal objects)
- `src/actor/` - Actor lifecycle management (spawn, destroy, registration)
- `src/property/` - Property replication and serialization
- `src/connection/` - Client connection management and authentication
- `src/rpc/` - Remote procedure call handling
- `src/relevancy/` - Network relevancy and interest management
- `src/generated/` - Generated code from the USpacetimeDBCodeGenerator
- `deploy.sh` - Deployment script for Linux/macOS
- `deploy.bat` - Deployment script for Windows

## Core Systems

### Object System
The object system replicates Unreal Engine's actor model within SpacetimeDB, handling:
- Actor spawning and initialization
- Object lifecycle states (Initializing, Active, PendingKill, Destroyed)
- Class hierarchies and inheritance
- Owner/component relationships

### Property System
The property system provides robust property management:
- Support for primitive types (bool, int, float, string, etc.)
- Structured types (Vector, Rotator, Transform, etc.)
- Reference types (object references, class references)
- Container types (arrays, maps, sets)
- Automatic serialization/deserialization

### RPC System
The RPC module enables bidirectional function calls:
- Client-to-server function calls (via SpacetimeDB reducers)
- Server-to-client callbacks (via subscription updates)
- Multicast RPC support for broadcasting to multiple clients
- Owner-only RPC targeting for secure operations
- Comprehensive error handling and logging

### Relevancy System
The relevancy system optimizes network usage:
- Multiple relevancy strategies (Always Relevant, Owner-Only, Distance-Based, Zone-Based)
- Update frequency control based on priority
- Automatic integration with property replication
- Zone management for logical grouping

## Integration with CustomServerModule

The ServerModule now integrates with the CustomServerModule to separate game-specific functionality from the core replication system:

```rust
// Import custom game functions
extern crate custom_server_module;
pub use custom_server_module as game;
```

This allows ServerModule code to access game-specific functions like:

```rust
// In a ServerModule reducer
let object_id = game::spawn_sphere(ctx, &params);
```

## Deployment

The ServerModule includes deployment scripts to easily publish your module to a SpacetimeDB instance.

### Using deploy.sh (Linux/macOS)

```bash
# Make the script executable
chmod +x deploy.sh

# Deploy with default settings (database: unreal_game, host: localhost:3000)
./deploy.sh

# Deploy with custom database name
./deploy.sh my_game_db

# Deploy with custom database name and host
./deploy.sh my_game_db my-server.spacetimedb.com:3000
```

### Using deploy.bat (Windows)

```batch
# Deploy with default settings (database: unreal_game, host: localhost:3000)
deploy.bat

# Deploy with custom database name
deploy.bat my_game_db

# Deploy with custom database name and host
deploy.bat my_game_db my-server.spacetimedb.com:3000
```

### Deployment Process

The deployment scripts:
1. Check for the SpacetimeDB CLI installation
2. Build the CustomServerModule (dependency)
3. Build the ServerModule
4. Create the database if it doesn't exist
5. Publish the module to the specified SpacetimeDB instance

### Prerequisites

- SpacetimeDB CLI (`spacetime`) must be installed
  - Install with: `cargo install spacetime`
- Rust and Cargo must be installed
- SpacetimeDB server must be running at the specified host

## Usage

The ServerModule is designed to be used as a backend for Unreal Engine games, providing a complete replacement for UE's native replication system. It connects to the ClientModule through SpacetimeDB, offering:

1. Actor spawning and replication
2. Property synchronization 
3. RPC system for game logic
4. Optimization through relevancy systems

This module is automatically compiled and deployed to SpacetimeDB through the build scripts in the plugin. 