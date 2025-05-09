//! # Actor Spawning
//!
//! Provides functions for spawning and initializing actors in the game world.

use spacetimedb::{ReducerContext, Identity};
use crate::actor::{ActorId, ActorInfo, ActorLifecycleState, ActorTransform};

/// Counter for generating unique actor IDs
static mut NEXT_ACTOR_ID: ActorId = 1000; // Start at 1000 to leave room for special IDs

/// Generates a unique actor ID
pub fn generate_actor_id() -> ActorId {
    unsafe {
        let id = NEXT_ACTOR_ID;
        NEXT_ACTOR_ID += 1;
        id
    }
}

/// Main reducer for spawning actors - exposed to clients
#[spacetimedb::reducer]
pub fn spawn_actor(
    ctx: &ReducerContext, 
    class_id: u32, 
    actor_name: String,
    position: (f32, f32, f32),
    rotation: (f32, f32, f32, f32),
    scale: (f32, f32, f32),
    initial_properties: Vec<(String, String)>  // Simple string-based property values for initialization
) -> ActorId {
    // Only allow clients to spawn actors if authorized
    if !crate::connection::auth::can_spawn_actor(ctx, class_id) {
        log::warn!("Client {:?} attempted to spawn actor of class {} without permission", 
                   ctx.sender, class_id);
        return 0; // Return 0 to indicate failure
    }
    
    // Generate new actor ID
    let actor_id = generate_actor_id();
    log::info!("Spawning actor: class={}, name={}, id={}", class_id, actor_name, actor_id);
    
    // Create actor base info
    ctx.db.actor_info().insert(ActorInfo {
        actor_id,
        class_id,
        actor_name,
        owner_identity: ctx.sender, // Set the spawning client as owner
        state: ActorLifecycleState::Spawning,
        created_at: ctx.timestamp,
        hidden: false,
    });
    
    // Set transform
    ctx.db.actor_transform().insert(ActorTransform {
        actor_id,
        pos_x: position.0,
        pos_y: position.1,
        pos_z: position.2,
        rot_x: rotation.0,
        rot_y: rotation.1,
        rot_z: rotation.2,
        rot_w: rotation.3,
        scale_x: scale.0,
        scale_y: scale.1,
        scale_z: scale.2,
    });
    
    // Set initial properties
    if !initial_properties.is_empty() {
        set_initial_properties(ctx, actor_id, initial_properties);
    }
    
    // Set up components if needed
    initialize_default_components(ctx, actor_id, class_id);
    
    // Mark actor as active
    if let Some(mut actor) = ctx.db.actor_info().filter_by_actor_id(&actor_id).first() {
        actor.state = ActorLifecycleState::Active;
        ctx.db.actor_info().update(&actor);
    }
    
    // Return the created actor ID
    actor_id
}

/// Spawn an actor with the server as the owner (system actor)
#[spacetimedb::reducer]
pub fn spawn_system_actor(
    ctx: &ReducerContext, 
    class_id: u32, 
    actor_name: String,
    position: (f32, f32, f32),
    rotation: (f32, f32, f32, f32),
    scale: (f32, f32, f32),
    initial_properties: Vec<(String, String)>
) -> ActorId {
    // Only authorized clients or the system can spawn system actors
    if !crate::connection::auth::is_admin(ctx) {
        log::warn!("Unauthorized client attempted to spawn system actor: {:?}", ctx.sender);
        return 0;
    }
    
    // Generate new actor ID
    let actor_id = generate_actor_id();
    log::info!("Spawning system actor: class={}, name={}, id={}", class_id, actor_name, actor_id);
    
    // Create actor base info (with no owner)
    ctx.db.actor_info().insert(ActorInfo {
        actor_id,
        class_id,
        actor_name,
        owner_identity: None, // System actor has no owner
        state: ActorLifecycleState::Spawning,
        created_at: ctx.timestamp,
        hidden: false,
    });
    
    // Set transform
    ctx.db.actor_transform().insert(ActorTransform {
        actor_id,
        pos_x: position.0,
        pos_y: position.1,
        pos_z: position.2,
        rot_x: rotation.0,
        rot_y: rotation.1,
        rot_z: rotation.2,
        rot_w: rotation.3,
        scale_x: scale.0,
        scale_y: scale.1,
        scale_z: scale.2,
    });
    
    // Set initial properties
    if !initial_properties.is_empty() {
        set_initial_properties(ctx, actor_id, initial_properties);
    }
    
    // Set up components if needed
    initialize_default_components(ctx, actor_id, class_id);
    
    // Mark actor as active
    if let Some(mut actor) = ctx.db.actor_info().filter_by_actor_id(&actor_id).first() {
        actor.state = ActorLifecycleState::Active;
        ctx.db.actor_info().update(&actor);
    }
    
    // Return the created actor ID
    actor_id
}

/// Spawn actor for a specific client (e.g., PlayerController, PlayerPawn)
#[spacetimedb::reducer]
pub fn spawn_player_actor(
    ctx: &ReducerContext,
    class_id: u32,
    player_identity: Identity,
    actor_name: String,
    position: (f32, f32, f32),
    rotation: (f32, f32, f32, f32),
    scale: (f32, f32, f32),
    initial_properties: Vec<(String, String)>
) -> ActorId {
    // Only the server or admin clients can spawn actors for specific players
    if !crate::connection::auth::is_admin(ctx) {
        log::warn!("Unauthorized client attempted to spawn player actor: {:?}", ctx.sender);
        return 0;
    }
    
    // Generate new actor ID
    let actor_id = generate_actor_id();
    log::info!("Spawning player actor: class={}, name={}, id={}, player={:?}", 
               class_id, actor_name, actor_id, player_identity);
    
    // Create actor base info with the specified player as owner
    ctx.db.actor_info().insert(ActorInfo {
        actor_id,
        class_id,
        actor_name,
        owner_identity: Some(player_identity),
        state: ActorLifecycleState::Spawning,
        created_at: ctx.timestamp,
        hidden: false,
    });
    
    // Set transform
    ctx.db.actor_transform().insert(ActorTransform {
        actor_id,
        pos_x: position.0,
        pos_y: position.1,
        pos_z: position.2,
        rot_x: rotation.0,
        rot_y: rotation.1,
        rot_z: rotation.2,
        rot_w: rotation.3,
        scale_x: scale.0,
        scale_y: scale.1,
        scale_z: scale.2,
    });
    
    // Set initial properties
    if !initial_properties.is_empty() {
        set_initial_properties(ctx, actor_id, initial_properties);
    }
    
    // Set up components if needed
    initialize_default_components(ctx, actor_id, class_id);
    
    // Mark actor as active
    if let Some(mut actor) = ctx.db.actor_info().filter_by_actor_id(&actor_id).first() {
        actor.state = ActorLifecycleState::Active;
        ctx.db.actor_info().update(&actor);
    }
    
    // Return the created actor ID
    actor_id
}

/// Helper function to set initial properties on a newly created actor
fn set_initial_properties(ctx: &ReducerContext, actor_id: ActorId, properties: Vec<(String, String)>) {
    for (name, value) in properties {
        // Convert the string value to the appropriate PropertyValue type
        // (For simplicity, we're using strings here, but in a real implementation you'd parse based on property type)
        let prop_value = crate::property::conversion::string_to_property_value(&value);
        
        // Insert the property
        ctx.db.actor_property().insert(crate::actor::ActorProperty {
            actor_id,
            property_name: name,
            value: prop_value,
            last_updated: ctx.timestamp,
        });
    }
}

/// Initialize default components for the actor based on its class
fn initialize_default_components(ctx: &ReducerContext, actor_id: ActorId, class_id: u32) {
    // In a real implementation, you'd have a system to define which components are needed for each class
    
    // For example, Characters might automatically need a movement component
    if class_id == 3 { // Character class
        // Create a movement component
        ctx.db.actor_component().insert(crate::actor::ActorComponent {
            component_id: generate_actor_id(), // Use the same ID generator for simplicity
            owner_actor_id: actor_id,
            component_class_id: 101, // Movement component class ID
            component_name: "CharacterMovement".to_string(),
            is_active: true,
        });
        
        // Create a mesh component
        ctx.db.actor_component().insert(crate::actor::ActorComponent {
            component_id: generate_actor_id(),
            owner_actor_id: actor_id,
            component_class_id: 102, // Mesh component class ID
            component_name: "Mesh".to_string(),
            is_active: true,
        });
    }
} 