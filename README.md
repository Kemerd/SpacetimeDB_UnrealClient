# SpacetimeDB Unreal Client Plugin

This plugin provides integration between Unreal Engine and SpacetimeDB, allowing games to connect to SpacetimeDB databases for multiplayer functionality and cloud-based game state.

## Features
- **Actor Replication**: Seamless integration with Unreal's actor and UObject system
- **Property System**: Robust property management with automatic serialization and synchronization
- **Object Lifecycle Management**: Complete lifecycle tracking for spawned objects
- **RPC System**: Bidirectional remote procedure calls between client and server
- **Cross-platform Support**: Works on Windows, macOS, and Linux
- **Seamless Integration**: Connect to SpacetimeDB instances with minimal code
- **High Performance**: Optimized Rust core with minimal overhead

## Architecture

The plugin uses a modular, layered architecture:

1. **Rust Core Modules**:
   - **SharedModule**: Common types, interfaces, and utilities used by both client and server
   - **ServerModule**: Server-side implementation that manages game state in SpacetimeDB
   - **ClientModule**: Client-side implementation that communicates with the server

2. **Module Structure**:
   - **Object System**: Manages UObject lifecycle, creation, and destruction
   - **Property System**: Handles property types, serialization, and synchronization
   - **Network Layer**: Manages connections, subscriptions, and data transfer
   - **RPC System**: Facilitates remote procedure calls between client and server

3. **Integration Layers**:
   - **FFI Layer**: C++ bindings generated via CXX that bridge Rust and C++
   - **Unreal-specific C++ Layer**: UE-friendly wrappers and subsystems
   - **Blueprint Integration**: Exposes functionality to Blueprint

## Core Modules

### Object System
The object system replicates Unreal Engine's actor model within SpacetimeDB, handling:
- Actor spawning and initialization
- Object lifecycle states (Initializing, Active, PendingKill, Destroyed)
- Class hierarchies and inheritance
- Owner/component relationships

### Property System
The property system provides a robust way to manage and replicate object properties:
- Support for primitive types (bool, int, float, string, etc.)
- Structured types (Vector, Rotator, Transform, etc.)
- Reference types (object references, class references)
- Container types (arrays, maps, sets)
- Automatic serialization/deserialization
- Change detection and efficient replication

### Network Layer
The network module handles all communication with the SpacetimeDB server:
- Connection establishment and management
- Client identification and authentication
- Data subscription and synchronization
- Event handling and callbacks

### RPC System
The RPC module enables bidirectional function calls between client and server:
- Client-to-server function calls (via SpacetimeDB reducers)
- Server-to-client callbacks (via subscription updates)
- Multicast RPC support for broadcasting to multiple clients
- Owner-only RPC targeting for secure operations
- Relevancy-based RPC delivery to optimize network usage
- JSON-based argument serialization and transport
- Comprehensive error handling and logging
- Type-safe function registration and invocation

## Prerequisites
- Unreal Engine 5.3+
- Rust (1.70+) - Install from [https://rustup.rs/](https://rustup.rs/)
- cxxbridge (`cargo install cxxbridge-cmd`)
- A C++ compiler compatible with your platform
  - Windows: Visual Studio 2022
  - macOS: Xcode
  - Linux: GCC or Clang
- SpacetimeDB CLI - For module development and testing

## Project Structure
```
/SpacetimeDB_UnrealClient/
├── /SharedModule/             # Shared types and interfaces
│   ├── Cargo.toml            # Shared module dependencies
│   └── /src/                 # Shared module source code
│       ├── lib.rs            # Module entry point
│       ├── types.rs          # Common data types
│       ├── property.rs       # Property system definitions
│       ├── object.rs         # Object system definitions
│       └── constants.rs      # Shared constants
│
├── /ServerModule/             # Server-side implementation
│   ├── Cargo.toml            # Server dependencies
│   └── /src/                 # Server module source code
│       ├── lib.rs            # Module entry point
│       ├── actor/            # Actor management
│       ├── object/           # Object management
│       ├── property/         # Property handling
│       ├── connection/       # Client connection management
│       ├── rpc/              # Remote procedure call functionality
│       ├── relevancy/        # Network relevancy system
│       └── reducer/          # SpacetimeDB reducers
│
├── /ClientModule/             # Client-side implementation
│   ├── Cargo.toml            # Client dependencies
│   └── /src/                 # Client module source code
│       ├── lib.rs            # Module entry point
│       ├── object/           # Client object management
│       ├── property/         # Client property handling
│       ├── net/              # Network communication
│       ├── rpc/              # RPC functionality
│       └── ffi.rs            # FFI definitions for C++
│
├── /Source/                   # Unreal Engine C++ code
│   └── /SpacetimeDB_UnrealClient/
│       ├── SpacetimeDB_UnrealClient.Build.cs
│       ├── SpacetimeDB_UnrealClient.h
│       └── SpacetimeDB_UnrealClient.cpp
│
└── SpacetimeDB_UnrealClient.uplugin  # Plugin definition
```

## How It Works

### Client-Server Communication
1. The client connects to a SpacetimeDB database using connection parameters (host, database name, auth token)
2. Upon connection, the client receives a unique client ID and subscribes to relevant tables
3. The server maintains the authoritative game state in SpacetimeDB tables
4. Changes to the database trigger events that are propagated to connected clients
5. Clients can request changes via reducers, which the server validates before applying

### Object Lifecycle
1. Objects are spawned via the server using the `spawn_object` reducer
2. The server assigns a unique ID and initializes the object with provided parameters
3. Object creation events are propagated to clients, which create local representations
4. Property updates flow between client and server based on replication rules
5. When an object is destroyed, clients receive a notification and clean up their local instances

### Property Replication
1. Properties are defined with types, replication conditions, and access rules
2. When a property changes on the server, it's serialized and sent to relevant clients
3. Clients deserialize property values and update their local object instances
4. Client-initiated property changes are sent to the server for validation and distribution
5. Special handling exists for transform properties (location, rotation, scale) to optimize network usage

### RPC System
1. Functions are registered with the server using the `register_rpc` API:
   ```rust
   // Server-side registration
   registry::register_rpc(
       "heal_player",           // Function name
       "PlayerCharacter",       // Class name
       RpcType::Server,         // RPC type (Server, Client, Multicast, OwnerOnly)
       true,                    // Is reliable
       Arc::new(|ctx, object_id, args_json| {
           // Function implementation
           // Return Result<String, RpcError>
       })
   );
   ```

2. Clients call server functions via a SpacetimeDB reducer:
   ```cpp
   // Client-side C++ call
   FRPCParams Params;
   Params.AddInt("HealAmount", 25);
   SpacetimeDB->CallServerFunction(PlayerID, "heal_player", Params);
   ```

3. The server processes the call through its dispatch system:
   - Validates the function exists
   - Verifies the object exists and is accessible
   - Executes the function handler with proper error handling
   - Returns results to the client

4. Server-to-client RPCs work through several targeting mechanisms:
   - `send_rpc_to_client`: Send to a specific client
   - `broadcast_rpc`: Send to all connected clients
   - `send_rpc_to_owner`: Send only to the owner of an object
   - `send_rpc_to_relevant_clients`: Send to clients who have visibility of an object
   - `send_rpc_to_zone`: Send to clients in a specific relevancy zone

5. All RPC calls are logged for debugging and analytics

## Usage

### Connecting to SpacetimeDB

```cpp
USpacetimeDBSubsystem* SpacetimeDB = GetGameInstance()->GetSubsystem<USpacetimeDBSubsystem>();
SpacetimeDB->Connect("localhost:3000", "my_game_db");
```

### Spawning an Actor

```cpp
FSpawnParams SpawnParams;
SpawnParams.ClassName = "Character";
SpawnParams.Location = FVector(100.0f, 0.0f, 0.0f);
SpawnParams.Replicate = true;

FObjectID ActorID = SpacetimeDB->SpawnActor(SpawnParams);
```

### Updating Properties

```cpp
SpacetimeDB->SetActorProperty(ActorID, "Location", FVector(200.0f, 0.0f, 0.0f));
```

### Calling Server Functions

```cpp
FRPCParams Params;
Params.AddInt("Damage", 25);
SpacetimeDB->CallServerFunction(ActorID, "TakeDamage", Params);
```

### Registering for Client RPC Callbacks

```cpp
// Register handler for a client RPC
SpacetimeDB->RegisterRPCHandler("OnDamaged", this, &AMyCharacter::HandleDamageEvent);

// Implement the handler
void AMyCharacter::HandleDamageEvent(FObjectID SourceID, const FRPCParams& Params)
{
    int32 DamageAmount = Params.GetInt("Damage");
    FString DamageType = Params.GetString("DamageType");
    
    // Process the event
    PlayDamageEffects(DamageType);
    UpdateHealthUI(CurrentHealth);
}
```

### Handling Events

```cpp
// Bind to property change events
SpacetimeDB->OnPropertyChanged.AddDynamic(this, &AMyGameMode::HandlePropertyChanged);

// Bind to object events
SpacetimeDB->OnActorSpawned.AddDynamic(this, &AMyGameMode::HandleActorSpawned);
SpacetimeDB->OnActorDestroyed.AddDynamic(this, &AMyGameMode::HandleActorDestroyed);
```

## Configuration
The following settings can be configured in your project's settings:
- `SPACETIME_HOST`: The SpacetimeDB server address
- `SPACETIME_DBNAME`: The database name
- `SPACETIME_AUTH_TOKEN`: Authentication token (if required)
- `SPACETIME_MAX_OBJECTS`: Maximum number of tracked objects (default: 100,000)
- `SPACETIME_REPLICATION_INTERVAL`: Property replication interval in seconds (default: 0.1)

## Troubleshooting

If you encounter issues:

1. Check the connection status using `SpacetimeDB->IsConnected()`
2. Enable debug logging with `SpacetimeDB->SetLogLevel(ESpacetimeLogLevel::Debug)`
3. Verify network connectivity to your SpacetimeDB instance
4. Check for error callbacks via `SpacetimeDB->OnError`

## License
MIT License
