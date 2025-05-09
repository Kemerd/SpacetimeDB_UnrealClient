//! # Client-side Actor System
//!
//! Handles client-side actor management, including actor creation, destruction,
//! and transformation.
//! 
//! NOTE: This module is now a thin wrapper around the object module, which 
//! has been consolidated to handle both actors and non-actor objects through
//! a unified interface.

use stdb_shared::object::{ObjectId, SpawnParams};
use stdb_shared::types::*;
use stdb_shared::lifecycle::ObjectLifecycleState;

use log::{info};

// Re-export from shared for convenience
pub use stdb_shared::object::ObjectId;

/// Initialize the actor system
pub fn init() {
    info!("Initializing client-side actor system");
    // Just log, actual initialization happens in object module
    info!("Client-side actor system initialized");
}

/// Create a client-side actor (forwards to object::create_actor)
pub fn create_actor(
    class_name: &str,
    transform: Transform,
    owner_id: Option<ObjectId>,
    replicates: bool,
) -> Result<ObjectId, String> {
    crate::object::create_actor(class_name, transform, owner_id, replicates)
}

/// Get an actor by ID (forwards to object::get_object with validation)
pub fn get_actor(actor_id: ObjectId) -> Option<crate::object::ClientObject> {
    let obj = crate::object::get_object(actor_id)?;
    if obj.is_actor() {
        Some(obj)
    } else {
        None
    }
}

/// Check if an object ID is an actor (forwards to object::is_actor)
pub fn is_actor(object_id: ObjectId) -> bool {
    crate::object::is_actor(object_id)
}

/// Update an actor's transform (forwards to object::update_transform)
pub fn update_transform(
    actor_id: ObjectId,
    location: Option<Vector3>,
    rotation: Option<Quat>,
    scale: Option<Vector3>,
) -> Result<(), String> {
    crate::object::update_transform(actor_id, location, rotation, scale)
}

/// Update an actor's lifecycle state (forwards to object::update_lifecycle_state)
pub fn update_lifecycle_state(
    actor_id: ObjectId,
    state: ObjectLifecycleState,
) -> Result<(), String> {
    crate::object::update_lifecycle_state(actor_id, state)
}

/// Destroy an actor (forwards to object::destroy_object)
pub fn destroy_actor(actor_id: ObjectId) -> Result<(), String> {
    crate::object::destroy_object(actor_id)
}

/// Add a component to an actor (forwards to object::add_component)
pub fn add_component(
    actor_id: ObjectId,
    component_id: ObjectId,
) -> Result<(), String> {
    crate::object::add_component(actor_id, component_id)
}

/// Remove a component from an actor (forwards to object::remove_component)
pub fn remove_component(
    actor_id: ObjectId,
    component_id: ObjectId,
) -> Result<(), String> {
    crate::object::remove_component(actor_id, component_id)
}

/// Get all components attached to an actor (forwards to object::get_components)
pub fn get_components(actor_id: ObjectId) -> Result<Vec<ObjectId>, String> {
    crate::object::get_components(actor_id)
}

/// Get all actors in the world (forwards to object::get_all_actors)
pub fn get_all_actors() -> Vec<crate::object::ClientObject> {
    crate::object::get_all_actors()
} 