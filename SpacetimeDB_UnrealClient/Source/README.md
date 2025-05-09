# Source

This directory contains the Unreal Engine C++ code for the SpacetimeDB Unreal Client Plugin. It provides the interface between Unreal Engine and the Rust modules through FFI.

## Purpose

The Source directory contains:
1. Unreal Engine plugin definitions and build configuration
2. C++ wrapper classes for the Rust functionality
3. Blueprint-exposed interfaces for game developers
4. Callback handling for events from SpacetimeDB
5. Integration with Unreal Engine's subsystems

## Structure

- `SpacetimeDB_UnrealClient.Build.cs` - UE build system configuration
- `SpacetimeDB_UnrealClient.h` - Main header file for the plugin
- `SpacetimeDB_UnrealClient.cpp` - Main implementation file for the plugin

### Core Components

- **USpacetimeDBSubsystem**: The primary interface for game code
- **FSpacetimeDB**: Internal implementation of the SpacetimeDB client
- **USpacetimeDBCodeGenerator**: Editor utility for generating Rust code

## USpacetimeDBSubsystem

The `USpacetimeDBSubsystem` is a Game Instance Subsystem that provides the main interface for game code to interact with SpacetimeDB. It offers:

- Connection management: `Connect()`, `Disconnect()`, `IsConnected()`
- Actor management: `SpawnActor()`, `DestroyActor()`
- Property management: `SetActorProperty()`, `GetActorProperty()`
- RPC system: `CallServerFunction()`, `RegisterRPCHandler()`
- Event handling: `OnActorSpawned`, `OnActorDestroyed`, `OnPropertyChanged`
- Relevancy system: `SetActorRelevancy()`, `CreateZone()`, etc.

## FSpacetimeDB

The `FSpacetimeDB` class is an internal implementation that handles:

- FFI communication with the Rust ClientModule
- Memory management for C++/Rust boundary
- Callback handling and marshaling
- Thread safety between Rust and Unreal Engine

## USpacetimeDBCodeGenerator

The `USpacetimeDBCodeGenerator` is an Editor Subsystem that:

- Scans UE classes to find replicable classes
- Generates Rust code for class registration
- Maps UE component hierarchies to Rust
- Creates property definitions for replicable properties

## Integration with Rust

The C++ code communicates with the Rust modules through FFI defined in the ClientModule's `ffi.rs`. It loads the compiled Rust dynamic library at runtime and interacts with it through the defined FFI functions.

## Usage Example

```cpp
// Get the SpacetimeDB subsystem
USpacetimeDBSubsystem* SpacetimeDB = GetGameInstance()->GetSubsystem<USpacetimeDBSubsystem>();

// Connect to SpacetimeDB
SpacetimeDB->Connect("localhost:3000", "my_game_db", "auth_token");

// Register for events
SpacetimeDB->OnActorSpawned.AddDynamic(this, &AMyGameMode::HandleActorSpawned);

// Spawn an actor
FSpawnParams SpawnParams;
SpawnParams.ClassName = "Character";
SpawnParams.Location = FVector(100.0f, 0.0f, 0.0f);
FObjectID ActorID = SpacetimeDB->SpawnActor(SpawnParams);

// Call a server function
FRPCParams Params;
Params.AddInt("Health", 100);
SpacetimeDB->CallServerFunction(ActorID, "SetHealth", Params);

// Update a property
SpacetimeDB->SetActorProperty(ActorID, "Location", FVector(200.0f, 0.0f, 0.0f));
```

## Blueprint Support

The plugin provides extensive Blueprint support through:

- Blueprint callable functions
- Blueprint assignable delegates for events
- Blueprint structs for parameter passing
- Blueprint read/write properties

This allows non-C++ developers to use the full functionality of SpacetimeDB integration. 