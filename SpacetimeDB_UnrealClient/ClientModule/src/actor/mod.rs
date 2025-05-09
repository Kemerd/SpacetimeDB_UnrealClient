//! # Client-side Actor System
//!
//! Handles client-side actor management, including actor creation, destruction,
//! and transformation.

use stdb_shared::object::{ObjectId, ObjectLifecycleState, SpawnParams};
use stdb_shared::property::PropertyValue;
use stdb_shared::types::*;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use log::{debug, error, info, warn};

// Re-export from shared for convenience
pub use stdb_shared::object::ObjectId;

/// Client-side actor representation
#[derive(Debug, Clone)]
pub struct ClientActor {
    /// Unique object ID
    pub id: ObjectId,
    
    /// Class name
    pub class_name: String,
    
    /// Owner ID (if any)
    pub owner_id: Option<ObjectId>,
    
    /// Current lifecycle state
    pub state: ObjectLifecycleState,
    
    /// Actor transform
    pub transform: Transform,
    
    /// Whether this actor replicates
    pub replicates: bool,
    
    /// Components attached to this actor
    pub components: Vec<ObjectId>,
}

/// Global registry of client actors
static CLIENT_ACTORS: Lazy<Mutex<HashMap<ObjectId, ClientActor>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Initialize the actor system
pub fn init() {
    info!("Initializing client-side actor system");
    // Nothing specific to initialize yet
    info!("Client-side actor system initialized");
}

/// Create a client-side actor
pub fn create_actor(
    object_id: ObjectId,
    class_name: &str,
    transform: Transform,
    owner_id: Option<ObjectId>,
    replicates: bool,
) -> Result<ClientActor, String> {
    let actor = ClientActor {
        id: object_id,
        class_name: class_name.to_string(),
        owner_id,
        state: ObjectLifecycleState::Initializing,
        transform,
        replicates,
        components: Vec::new(),
    };
    
    // Register the actor
    {
        let mut actors = CLIENT_ACTORS.lock().unwrap();
        actors.insert(object_id, actor.clone());
    }
    
    // Return the new actor
    Ok(actor)
}

/// Get an actor by ID
pub fn get_actor(actor_id: ObjectId) -> Option<ClientActor> {
    let actors = CLIENT_ACTORS.lock().unwrap();
    actors.get(&actor_id).cloned()
}

/// Check if an object ID is an actor
pub fn is_actor(object_id: ObjectId) -> bool {
    let actors = CLIENT_ACTORS.lock().unwrap();
    actors.contains_key(&object_id)
}

/// Update an actor's transform
pub fn update_transform(
    actor_id: ObjectId,
    location: Option<Vector3>,
    rotation: Option<Quat>,
    scale: Option<Vector3>,
) -> Result<(), String> {
    let mut actors = CLIENT_ACTORS.lock().unwrap();
    
    if let Some(actor) = actors.get_mut(&actor_id) {
        // Update location if provided
        if let Some(loc) = location {
            actor.transform.location = loc;
            
            // Also update the location property in the object system
            let loc_value = PropertyValue::Vector(loc);
            crate::property::cache_property_value(actor_id, "Location", loc_value);
        }
        
        // Update rotation if provided
        if let Some(rot) = rotation {
            actor.transform.rotation = rot;
            
            // Also update the rotation property in the object system
            let rot_value = PropertyValue::Quat(rot);
            crate::property::cache_property_value(actor_id, "Rotation", rot_value);
        }
        
        // Update scale if provided
        if let Some(scl) = scale {
            actor.transform.scale = scl;
            
            // Also update the scale property in the object system
            let scale_value = PropertyValue::Vector(scl);
            crate::property::cache_property_value(actor_id, "Scale", scale_value);
        }
        
        Ok(())
    } else {
        Err(format!("Actor {} not found", actor_id))
    }
}

/// Update an actor's lifecycle state
pub fn update_lifecycle_state(
    actor_id: ObjectId,
    state: ObjectLifecycleState,
) -> Result<(), String> {
    let mut actors = CLIENT_ACTORS.lock().unwrap();
    
    if let Some(actor) = actors.get_mut(&actor_id) {
        actor.state = state;
        
        // If destroyed, remove from registry
        if state == ObjectLifecycleState::Destroyed {
            drop(actors); // Release lock before calling destroy_actor
            destroy_actor(actor_id)?;
        }
        
        Ok(())
    } else {
        Err(format!("Actor {} not found", actor_id))
    }
}

/// Destroy an actor
pub fn destroy_actor(actor_id: ObjectId) -> Result<(), String> {
    // Remove from actor registry
    {
        let mut actors = CLIENT_ACTORS.lock().unwrap();
        actors.remove(&actor_id);
    }
    
    // Clean up properties
    crate::property::clear_object_cache(actor_id);
    
    info!("Actor {} destroyed", actor_id);
    Ok(())
}

/// Add a component to an actor
pub fn add_component(
    actor_id: ObjectId,
    component_id: ObjectId,
) -> Result<(), String> {
    let mut actors = CLIENT_ACTORS.lock().unwrap();
    
    if let Some(actor) = actors.get_mut(&actor_id) {
        if !actor.components.contains(&component_id) {
            actor.components.push(component_id);
            Ok(())
        } else {
            Err(format!("Component {} already attached to actor {}", component_id, actor_id))
        }
    } else {
        Err(format!("Actor {} not found", actor_id))
    }
}

/// Remove a component from an actor
pub fn remove_component(
    actor_id: ObjectId,
    component_id: ObjectId,
) -> Result<(), String> {
    let mut actors = CLIENT_ACTORS.lock().unwrap();
    
    if let Some(actor) = actors.get_mut(&actor_id) {
        actor.components.retain(|&id| id != component_id);
        Ok(())
    } else {
        Err(format!("Actor {} not found", actor_id))
    }
}

/// Get all components attached to an actor
pub fn get_components(actor_id: ObjectId) -> Result<Vec<ObjectId>, String> {
    let actors = CLIENT_ACTORS.lock().unwrap();
    
    if let Some(actor) = actors.get(&actor_id) {
        Ok(actor.components.clone())
    } else {
        Err(format!("Actor {} not found", actor_id))
    }
}

/// Get all actors in the world
pub fn get_all_actors() -> Vec<ClientActor> {
    let actors = CLIENT_ACTORS.lock().unwrap();
    actors.values().cloned().collect()
} 