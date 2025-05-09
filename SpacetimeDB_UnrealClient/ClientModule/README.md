# ClientModule

This module provides the client-side implementation of the SpacetimeDB Unreal Client Plugin. It communicates with the ServerModule through SpacetimeDB and provides an FFI interface for the Unreal Engine C++ code.

## Purpose

The ClientModule handles:
1. Client-side object representation and management
2. Communication with the SpacetimeDB server
3. Property tracking and application
4. Client-side prediction (optional)
5. FFI interface for Unreal Engine C++ integration

## Structure

- `src/lib.rs` - Module entry point with initialization
- `src/object/` - Client object representation and management
- `src/property/` - Client property handling and tracking
- `src/net/` - Network communication with SpacetimeDB
- `src/rpc/` - RPC functionality for client-side handling
- `src/ffi.rs` - FFI interface for C++ integration

## Core Systems

### Object System
The client object system maintains a local representation of server objects:
- Tracks local objects mirroring server objects
- Manages object creation and destruction
- Handles object properties and components

### Property System
The property system on the client side:
- Receives property updates from the server
- Applies updates to local objects
- Tracks property changes for client-side prediction
- Serializes property values for sending to the server

### Network System
The network module handles communication with SpacetimeDB:
- Connection establishment and management
- Subscription to relevant tables
- Handling of incoming messages and events
- Sending requests to the server

### RPC System
The RPC system on the client side:
- Sends RPC requests to the server
- Handles incoming RPC calls from the server
- Manages RPC callbacks and responses

### FFI Interface
The FFI interface is the bridge between Rust and C++:
- Exposes client functionality to C++
- Provides callback mechanisms for events
- Handles serialization between Rust and C++ types
- Enables Unreal Engine integration

## Integration with Unreal Engine

The ClientModule is designed to integrate with Unreal Engine through the FFI interface in `ffi.rs`. This interface is used by the C++ USpacetimeDBSubsystem to:

1. Connect to SpacetimeDB
2. Spawn and manage actors
3. Update properties
4. Call server functions
5. Receive and handle events

The C++ code uses this interface to create a seamless integration between SpacetimeDB and Unreal Engine, providing a UE-friendly API for developers.

## Usage

The ClientModule is compiled into a dynamic library (`stdb_client`) that is loaded by the UE plugin. It provides the runtime functionality for:

1. Actor replication from server to client
2. Property updates and synchronization
3. RPC calls in both directions
4. Event handling and callbacks

This module is automatically compiled by the build scripts in the plugin. 