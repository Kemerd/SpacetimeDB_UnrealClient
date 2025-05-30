# SpacetimeDB Unreal Client Plugin

This plugin provides integration between Unreal Engine and SpacetimeDB, allowing games to connect to SpacetimeDB databases for multiplayer functionality and cloud-based game state.

## Features
- **Actor Replication**: Seamless integration with Unreal's actor and UObject system
- **Property System**: Robust property management with automatic serialization and synchronization
- **Object Lifecycle Management**: Complete lifecycle tracking for spawned objects
- **RPC System**: Bidirectional remote procedure calls between client and server
- **Network Relevancy**: Optimization system to send updates only to clients that need them
- **Cross-platform Support**: Works on Windows, macOS, and Linux
- **Seamless Integration**: Connect to SpacetimeDB instances with minimal code
- **High Performance**: Optimized Rust core with minimal overhead
- **Game-Specific Customization**: Extend with your own game functions in CustomServerModule

## Architecture

The plugin uses a modular, layered architecture:

1. **Rust Core Modules**:
   - **SharedModule**: Common types, interfaces, and utilities used by both client and server
   - **ServerModule**: Server-side implementation that manages game state in SpacetimeDB
   - **ClientModule**: Client-side implementation that communicates with the server
   - **CustomServerModule**: Game-specific functionality and example game functions

2. **Module Structure**:
   - **Object System**: Manages UObject lifecycle, creation, and destruction
   - **Property System**: Handles property types, serialization, and synchronization
   - **Network Layer**: Manages connections, subscriptions, and data transfer
   - **RPC System**: Facilitates remote procedure calls between client and server
   - **Relevancy System**: Determines which objects are relevant to which clients
   - **Game Functions**: Custom game-specific functions in the CustomServerModule

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
- Automatic property definition synchronization between server and client
- Runtime property inspection and discovery
- JSON import/export support for property definitions

The property system uses an efficient cache strategy:
1. Property definitions are automatically synchronized from server to client
2. Clients can query property information by class or property name
3. A staging cache holds property values that haven't been assigned to objects yet
4. Values are transferred from the staging cache to objects upon creation
5. Property changes are tracked for efficient network replication

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

### Network Relevancy System
The relevancy system optimizes network usage by controlling which objects and property updates are sent to each client:
- Multiple relevancy strategies:
  - **Always Relevant**: Objects that all clients need to know about (game state, global actors)
  - **Owner-Only**: Objects that only the owner client needs (player-specific inventory, abilities)
  - **Distance-Based**: Objects only relevant to clients within a certain distance (spatial optimization)
  - **Zone-Based**: Objects only relevant to clients in the same "zone" (rooms, areas, levels)
  - **Custom**: Custom logic for specialized relevancy determination
- Update frequency control based on priority and importance
- Automatic integration with property replication system
- Zone management for logical grouping of objects and clients
- Distance-based relevancy using spatial partitioning for efficiency

### CustomServerModule
The CustomServerModule provides game-specific functionality built on top of the core replication system:
- Separation of game logic from core replication functionality
- Example functions for common game operations:
  - `spawn_sphere`: Spawn a sphere with customizable properties
  - `spawn_cube`: Spawn a cube with size, rotation, and material options
  - `teleport_actor`: Teleport an actor to a new position
  - `change_color`: Change the color of an actor with optional animation
  - `spawn_projectile`: Spawn a projectile with velocity and attributes
- Easy extensibility for your own game functions
- Integration with the ServerModule through the `game` namespace

#### Using CustomServerModule
The CustomServerModule is integrated into the ServerModule:
```rust
// In ServerModule/src/lib.rs
extern crate custom_server_module;
pub use custom_server_module as game;
```

This allows ServerModule code to call game functions:
```rust
// Example of calling a game function from the ServerModule
let sphere_id = game::spawn_sphere(ctx, &params);
```

From the C++ side, these functions are exposed through SpacetimeDB's RPC system:
```cpp
// Example of calling a game function from C++
FSpawnSphereParams Params;
Params.Position = FVector(100.0f, 0.0f, 50.0f);
Params.Radius = 50.0f;
Params.Color = FColor::Red;
Params.PhysicsEnabled = true;
SpacetimeDB->CallServerFunction("spawn_sphere", Params.ToJson());
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

## Code Generation System

The plugin includes a powerful code generation system that eliminates the need for hardcoding Unreal Engine classes and components in Rust. This system scans your project's UE classes and automatically generates Rust code to register them with SpacetimeDB.

### SpacetimeDBCodeGenerator

The `USpacetimeDBCodeGenerator` is an Editor Subsystem that analyzes your project's class hierarchy and generates:

1. **Class Registry**: Rust code that registers all relevant UE classes with SpacetimeDB, preserving class hierarchies, replication settings, and other metadata
2. **Component Mappings**: Rust code that sets up default components for each actor class based on their CDO (Class Default Object)
3. **Property Definitions**: Automatic registration of replicated properties with appropriate type mapping

### Benefits

- **Dynamic Class Discovery**: No need to manually register every class - the generator finds them automatically
- **Component Inheritance**: Captures the exact component setup of your actors as defined in the editor
- **Type Safety**: Ensures consistent class and component IDs between C++ and Rust
- **Build-Time Security**: Class registration happens at build time, not runtime, preventing unauthorized class addition

### How to Use

1. **Access the Generator**: The generator is available as an Editor Subsystem
   ```cpp
   USpacetimeDBCodeGenerator* CodeGen = GEditor->GetEditorSubsystem<USpacetimeDBCodeGenerator>();
   ```

2. **Generate Class Registry**:
   ```cpp
   FString OutputPath = FPaths::ProjectPluginsDir() / "SpacetimeDB_UnrealClient/ServerModule/src/generated/class_registry.rs";
   CodeGen->GenerateRustClassRegistry(OutputPath);
   ```

3. **Generate Component Mappings**:
   ```cpp
   FString OutputPath = FPaths::ProjectPluginsDir() / "SpacetimeDB_UnrealClient/ServerModule/src/generated/component_mappings.rs";
   CodeGen->GenerateRustComponentMappings(OutputPath);
   ```

4. **When to Run**: Generate these files whenever:
   - You add new actor or component classes to your project
   - You change the default component setup of an actor
   - You add or modify replicated properties

### Integration with Server Module

The generated files are automatically included in the server module's initialization:

```rust
// In ServerModule/src/object/init.rs
pub fn initialize_class_system(ctx: &ReducerContext) {
    // Register core classes (UObject, AActor, etc.)
    register_core_classes(ctx);
    
    // Register all generated classes from the code generator
    crate::generated::register_all_classes(ctx);
    
    // Register properties for all classes
    crate::generated::register_all_properties(ctx);
}
```

The component system also uses the generated mappings:

```rust
// In ServerModule/src/actor/spawn.rs
fn initialize_default_components(ctx: &ReducerContext, actor_id: ActorId, class_id: u32) {
    // Use generated component mappings
    crate::generated::initialize_components_for_class(ctx, actor_id, class_id);
}
```

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
├── /CustomServerModule/       # Game-specific functionality
│   ├── Cargo.toml            # Custom server module dependencies
│   └── /src/                 # Custom server module source code
│       ├── lib.rs            # Module entry point
│       └── functions/        # Game-specific functions
│           ├── spawn_sphere.rs     # Example function to spawn a sphere
│           ├── spawn_cube.rs       # Example function to spawn a cube
│           ├── teleport_actor.rs   # Example function to teleport an actor
│           ├── change_color.rs     # Example function to change actor color
│           └── spawn_projectile.rs # Example function to spawn projectiles
│
├── /Source/                   # Unreal Engine C++ code
│   └── /SpacetimeDB_UnrealClient/
│       ├── SpacetimeDB_UnrealClient.Build.cs
│       ├── SpacetimeDB_UnrealClient.h
│       └── SpacetimeDB_UnrealClient.cpp
│
└── SpacetimeDB_UnrealClient.uplugin  # Plugin definition
```

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

FObjectID ObjectID = SpacetimeDB->SpawnActor(SpawnParams);
```

### Updating Properties

```cpp
SpacetimeDB->SetActorProperty(ObjectID, "Location", FVector(200.0f, 0.0f, 0.0f));
```

### Calling Server Functions

```cpp
FRPCParams Params;
Params.AddInt("Damage", 25);
SpacetimeDB->CallServerFunction(ObjectID, "TakeDamage", Params);
```

### Using CustomServerModule Functions

```cpp
// Spawn a sphere using the CustomServerModule function
FSpawnSphereParams SphereParams;
SphereParams.Position = FVector(100.0f, 0.0f, 50.0f);
SphereParams.Radius = 50.0f;
SphereParams.Color = FColor::Red;
SphereParams.PhysicsEnabled = true;
FObjectID SphereID = SpacetimeDB->CallServerFunction("spawn_sphere", SphereParams.ToJson());

// Change the color of an actor
FChangeColorParams ColorParams;
ColorParams.ActorID = SphereID;
ColorParams.Color = FColor::Blue;
ColorParams.AnimateTransition = true;
ColorParams.TransitionDuration = 2.0f;
SpacetimeDB->CallServerFunction("change_color", ColorParams.ToJson());
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

### Managing Network Relevancy

```cpp
// Set relevancy settings for an actor
FRelevancySettings Settings;
Settings.Level = ERelevancyLevel::DistanceBased;
Settings.MaxDistance = 1000.0f;
Settings.UpdateFrequency = EUpdateFrequency::Medium;
Settings.Priority = ENetworkPriority::Normal;
SpacetimeDB->SetActorRelevancy(ObjectID, Settings);

// Create a zone
uint32 ZoneID = SpacetimeDB->CreateZone("Dungeon_Level_1", true);

// Add actors to a zone
SpacetimeDB->AddActorToZone(ObjectID, ZoneID);

// Add player to a zone
SpacetimeDB->AddClientToZone(ClientID, ZoneID);

// Remove actor from a zone
SpacetimeDB->RemoveActorFromZone(ObjectID, ZoneID);
```

## Configuration
The following settings can be configured in your project's settings:
- `SPACETIME_HOST`: The SpacetimeDB server address
- `SPACETIME_DBNAME`: The database name
- `SPACETIME_AUTH_TOKEN`: Authentication token (if required)
- `SPACETIME_MAX_OBJECTS`: Maximum number of tracked objects (default: 100,000)
- `SPACETIME_REPLICATION_INTERVAL`: Property replication interval in seconds (default: 0.1)
- `SPACETIME_DEFAULT_RELEVANCY`: Default relevancy level for new objects (default: AlwaysRelevant)
- `SPACETIME_MAX_RELEVANCY_DISTANCE`: Maximum distance for distance-based relevancy (default: 10000.0)
- `SPACETIME_ZONE_LIMIT`: Maximum number of zones (default: 1000)

## Troubleshooting

If you encounter issues:

1. Check the connection status using `SpacetimeDB->IsConnected()`
2. Enable debug logging with `SpacetimeDB->SetLogLevel(ESpacetimeLogLevel::Debug)`
3. Verify network connectivity to your SpacetimeDB instance
4. Check for error callbacks via `SpacetimeDB->OnError`

## License
MIT License
