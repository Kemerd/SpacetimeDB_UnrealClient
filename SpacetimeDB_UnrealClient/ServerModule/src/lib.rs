//! # UnrealReplication Server Module
//! 
//! This module provides a complete replacement for Unreal Engine's native replication
//! system using SpacetimeDB. It handles actor spawning, destruction, property replication,
//! RPCs, and network relevancy.
//!
//! The system is organized into several sub-modules:
//! - `object`: Core UObject functionality (base for all Unreal objects)
//! - `actor`: Actor lifecycle management (spawn, destroy, registration)
//! - `property`: Property replication and serialization
//! - `connection`: Client connection management and authentication
//! - `rpc`: Remote procedure call handling
//! - `relevancy`: Network relevancy and interest management

use spacetimedb::{ReducerContext, Table};

// Module declarations
pub mod object;    // Base UObject functionality
pub mod actor;     // Actor-specific functionality
pub mod property;  // Property system
pub mod connection;  // Connection management
pub mod rpc;         // Remote procedure calls
pub mod relevancy;   // Network relevancy

// Re-export commonly used items
pub use object::{ObjectInstance, ObjectClass, ObjectId, ObjectTransform, ObjectComponent, ObjectProperty};
pub use stdb_shared::lifecycle::ObjectLifecycleState;
pub use actor::ActorId;  // ActorId is now a type alias for ObjectId
pub use property::{PropertyType, PropertyValue};
pub use connection::{ClientConnection, ConnectionState};
pub use rpc::{RpcType, RpcCall};
pub use relevancy::{RelevancyZone, VisibilityFlag};

/// Initialize the database and any required systems
#[spacetimedb::reducer(init)]
pub fn init(ctx: &ReducerContext) {
    log::info!("UnrealReplication server module initialized");
    
    // Initialize core UObject system
    object::class::initialize_object_classes(ctx);
    
    // Initialize actor system
    actor::init::initialize_world(ctx);
    
    // Set up any global game state
    log::info!("World initialization complete");
}

/// Handle client connection
#[spacetimedb::reducer(client_connected)]
pub fn client_connected(ctx: &ReducerContext) {
    log::info!("Client connected: {:?}", ctx.sender);
    
    // Register new client in the connection system
    connection::handlers::register_client(ctx);
}

/// Handle client disconnection
#[spacetimedb::reducer(client_disconnected)]
pub fn client_disconnected(ctx: &ReducerContext) {
    log::info!("Client disconnected: {:?}", ctx.sender);
    
    // Clean up client-owned actors and state
    connection::handlers::unregister_client(ctx);
}

// Public module-specific reducers are defined in their respective modules
// and will be accessible to clients without needing to be defined here 