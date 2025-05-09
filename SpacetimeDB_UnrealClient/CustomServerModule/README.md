# CustomServerModule

This module contains game-specific functionality and example game functions built on top of the core UnrealReplication ServerModule. It's designed to demonstrate how to create custom game logic while leveraging the SpacetimeDB replication system.

## Purpose

The CustomServerModule serves as:
1. A demonstration of how to implement game-specific functions
2. A template for your own game's server-side logic
3. A separation of concerns between the core replication system and game-specific code

## Structure

- `src/lib.rs` - Module entry point with initialization
- `src/functions/` - Directory containing all game-specific function implementations
  - `spawn_sphere.rs` - Example function for spawning sphere actors
  - `spawn_cube.rs` - Example function for spawning cube actors
  - `teleport_actor.rs` - Example function for teleporting actors
  - `change_color.rs` - Example function for changing actor colors
  - `spawn_projectile.rs` - Example function for spawning projectiles with velocity

## Example Functions

### Spawn Sphere
```rust
// Spawn a red sphere at (100, 0, 50) with radius 50 and physics enabled
let params = SpawnSphereParams {
    position: StdbVector3 { x: 100.0, y: 0.0, z: 50.0 },
    radius: 50.0,
    color: StdbColor { r: 255, g: 0, b: 0, a: 255 },
    physics_enabled: true,
};
spawn_sphere(ctx, &params);
```

### Spawn Cube
```rust
// Spawn a blue cube with custom properties
let params = SpawnCubeParams {
    position: StdbVector3 { x: 0.0, y: 0.0, z: 100.0 },
    rotation: StdbRotator { pitch: 0.0, yaw: 45.0, roll: 0.0 },
    size: StdbVector3 { x: 100.0, y: 50.0, z: 25.0 },
    color: StdbColor { r: 0, g: 0, b: 255, a: 255 },
    physics_enabled: true,
    material: None,
};
spawn_cube(ctx, &params);
```

### Teleport Actor
```rust
// Teleport an actor to a new position with effects
let params = TeleportActorParams {
    actor_id: ObjectId(1000),
    position: StdbVector3 { x: 500.0, y: 500.0, z: 100.0 },
    with_effects: true,
};
teleport_actor(ctx, &params);
```

## Usage

1. Import the CustomServerModule in your ServerModule:
```rust
extern crate custom_server_module;
pub use custom_server_module as game;
```

2. Call functions from your game code:
```rust
// In your ServerModule code or reducers
let sphere_id = game::spawn_sphere(ctx, &sphere_params);
```

3. Add your own game functions by creating new files in the `functions/` directory and adding them to `functions/mod.rs`.

## Extending

To add your own game functions:

1. Create a new file in `src/functions/` (e.g., `my_function.rs`)
2. Implement your function with the `#[spacetimedb::reducer]` attribute
3. Add your module to `src/functions/mod.rs`
4. Re-export your function in `src/functions/mod.rs`

This module is designed to be extended with your own game-specific functionality while keeping it separate from the core replication system. 