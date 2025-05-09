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
    // Get class details from the registry
    if let Some(actor_class) = ctx.db.object_class().filter_by_class_id(&class_id).first() {
        // Get parent class hierarchy to apply component inheritance
        let mut current_class_id = class_id;
        let mut class_hierarchy = Vec::new();
        
        // Build the class hierarchy from the current class up to the root
        while current_class_id != 0 {
            if let Some(class) = ctx.db.object_class().filter_by_class_id(&current_class_id).first() {
                class_hierarchy.push(current_class_id);
                current_class_id = class.parent_class_id;
            } else {
                break;
            }
        }
        
        log::debug!("Setting up components for actor {} of class {}", actor_id, actor_class.class_name);
        
        // Initialize components based on class type
        // In a production system, this would be loaded from a configuration table
        // that maps class IDs to required component class IDs
        match actor_class.class_name.as_str() {
            // Characters automatically get character movement and skeletal mesh components
            "Character" => {
                // Add CharacterMovementComponent
                add_component(ctx, actor_id, 21, "CharacterMovement");
                
                // Add SkeletalMeshComponent for the character mesh
                add_component(ctx, actor_id, 7, "CharacterMesh");
            },
            "Pawn" => {
                // Pawns get a movement component
                add_component(ctx, actor_id, 20, "MovementComponent");
                
                // Add a static mesh component for basic visual representation
                add_component(ctx, actor_id, 6, "PawnMesh");
            },
            "Actor" => {
                // Base actors get a scene component as their root
                add_component(ctx, actor_id, 3, "RootComponent");
            },
            "PlayerController" => {
                // No visual components for controllers
            },
            "AIController" => {
                // No visual components for controllers
            },
            // Default case - all actors should have at least a root component
            _ => {
                // All actors by default have a root scene component
                if actor_class.is_actor {
                    add_component(ctx, actor_id, 3, "RootComponent");
                }
            }
        }
        
        // Apply any custom component specifications (in a real system, this would
        // come from a database table or configuration file)
        apply_custom_components(ctx, actor_id, class_id);
    }
}

/// Helper function to add a component to an actor
fn add_component(ctx: &ReducerContext, actor_id: ActorId, component_class_id: u32, component_name: &str) {
    // Get the class to verify it exists and is a component
    if let Some(component_class) = ctx.db.object_class().filter_by_class_id(&component_class_id).first() {
        if !component_class.is_component {
            log::warn!("Attempted to add non-component class {} as component", component_class.class_name);
            return;
        }
        
        log::debug!("Adding component {} of type {} to actor {}", 
                   component_name, component_class.class_name, actor_id);
        
        // Create the component
        ctx.db.actor_component().insert(crate::actor::ActorComponent {
            component_id: generate_actor_id(), // Use the same ID generator for components
            owner_actor_id: actor_id,
            component_class_id,
            component_name: component_name.to_string(),
            is_active: true,
        });
    } else {
        log::warn!("Attempted to add component with invalid class ID: {}", component_class_id);
    }
}

/// Apply any custom component specifications from game-specific configurations
fn apply_custom_components(ctx: &ReducerContext, actor_id: ActorId, class_id: u32) {
    // In a production implementation, this would query a ComponentRequirements table
    // that maps class IDs to required component specifications
    
    // For now, we'll just add some additional game-specific components based on hardcoded rules
    // Future implementations should replace this with data-driven configuration
    
    // Example of checking for a specific class ID range for custom game classes
    if class_id >= 100 && class_id < 200 {
        // Game-specific actors might need particular components
        // For example: Add an inventory component to player-related classes
        add_component(ctx, actor_id, 50, "InventoryComponent");
    }
} 