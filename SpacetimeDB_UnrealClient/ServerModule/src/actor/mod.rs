//! # Actor Module
//! 
//! Handles actor lifecycle (spawning, destruction, registration) and core actor data structures.
//! This module provides the foundation for replicating Unreal Engine actors in SpacetimeDB.
//! 
//! This module now uses the consolidated object tables rather than maintaining separate actor-specific tables.

use spacetimedb::{ReducerContext, Table, Identity};
use stdb_shared::property::{PropertyType, PropertyValue};
use stdb_shared::lifecycle::ObjectLifecycleState;
use crate::object::{ObjectId, ObjectInstance, ObjectTransform, ObjectProperty, ObjectComponent, ObjectClass};

// Submodules
pub mod init;       // Actor world initialization
pub mod spawn;      // Actor spawning and creation
pub mod lifecycle;  // Actor lifecycle management
pub mod transform;  // Actor transform handling (position, rotation, scale)

// Export the ObjectId type as ActorId for backward compatibility and clarity
// This maintains the strong typing that distinguishes actors from other objects
// while using the shared object ID space
pub type ActorId = ObjectId;

/// Quick helper function to validate that an object is an actor
pub fn is_valid_actor(ctx: &ReducerContext, actor_id: ActorId) -> bool {
    ctx.db.object_instance()
        .filter_by_object_id(&actor_id)
        .first()
        .map(|obj| obj.is_actor)
        .unwrap_or(false)
}

/// Helper function to get an actor's transform
pub fn get_actor_transform(ctx: &ReducerContext, actor_id: ActorId) -> Option<ObjectTransform> {
    if !is_valid_actor(ctx, actor_id) {
        return None;
    }
    
    ctx.db.object_transform()
        .filter_by_object_id(&actor_id)
        .first()
        .cloned()
}

/// Helper function to get all components for an actor
pub fn get_actor_components(ctx: &ReducerContext, actor_id: ActorId) -> Vec<ObjectComponent> {
    if !is_valid_actor(ctx, actor_id) {
        return Vec::new();
    }
    
    ctx.db.object_component()
        .filter_by_owner_object_id(&actor_id)
        .collect::<Vec<_>>()
}

/// Helper function to get actor properties
pub fn get_actor_properties(ctx: &ReducerContext, actor_id: ActorId) -> Vec<ObjectProperty> {
    if !is_valid_actor(ctx, actor_id) {
        return Vec::new();
    }
    
    ctx.db.object_property()
        .filter_by_object_id(&actor_id)
        .collect::<Vec<_>>()
}

/// Helper function to get actor info
pub fn get_actor_info(ctx: &ReducerContext, actor_id: ActorId) -> Option<ObjectInstance> {
    let obj = ctx.db.object_instance()
        .filter_by_object_id(&actor_id)
        .first()
        .cloned();
    
    // Verify this is an actor, not just any object
    if let Some(obj) = obj {
        if obj.is_actor {
            return Some(obj);
        }
    }
    
    None
} 