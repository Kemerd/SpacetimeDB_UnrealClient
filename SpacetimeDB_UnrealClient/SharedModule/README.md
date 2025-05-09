# SharedModule

This module provides common types, interfaces, and utilities used by both the ClientModule and ServerModule. It ensures consistency between client and server implementations.

## Purpose

The SharedModule serves as:
1. A common interface between client and server code
2. A central repository for shared data types and structures
3. A mechanism to ensure type consistency across modules
4. A way to share constants and configuration values

## Structure

- `src/lib.rs` - Module entry point and exports
- `src/types.rs` - Common data types (Vector3, Rotator, Transform, Color, etc.)
- `src/property.rs` - Property system definitions
- `src/object.rs` - Object system definitions
- `src/rpc.rs` - RPC system shared types
- `src/lifecycle.rs` - Object lifecycle states and transitions
- `src/connection.rs` - Connection-related types
- `src/constants.rs` - Shared constants and configuration values

## Core Components

### Common Data Types
The SharedModule defines common data types used throughout the system:
- `StdbVector3` - 3D vector type
- `StdbRotator` - Rotation type
- `StdbTransform` - Combined location, rotation, and scale
- `StdbColor` - RGBA color representation
- And many more UE-specific types

### Property System Types
Property-related types that ensure consistent property handling:
- `PropertyType` - Enumeration of supported property types
- `PropertyValue` - Variant type for property values
- `PropertyDefinition` - Property metadata
- `ReplicationCondition` - When properties should replicate

### Object System Types
Object-related types for the object system:
- `ObjectId` - Unique identifier for objects
- `SpawnParams` - Parameters for spawning objects
- `ObjectLifecycleState` - States in an object's lifecycle

### RPC System Types
Types for the RPC system:
- `RpcType` - Type of RPC (Server, Client, Multicast, etc.)
- `RpcCall` - Structure for RPC calls
- `RpcResponse` - Structure for RPC responses
- `RpcStatus` - Status of an RPC call
- `RpcError` - Error types for RPC failures

## Usage

The SharedModule is imported by both the ClientModule and ServerModule:

```rust
// In ClientModule or ServerModule
use stdb_shared::types::StdbVector3;
use stdb_shared::property::{PropertyType, PropertyValue};
use stdb_shared::object::ObjectId;
```

This ensures that both modules are using the same type definitions and interfaces.

## Extending

When adding new shared functionality:

1. Determine which file the new type belongs in
2. Add the type definition
3. Ensure it's properly exported in the module hierarchy
4. Add any required serialization/deserialization implementations

The SharedModule is critical for maintaining consistency between client and server implementations. 