//! # Actor Initialization
//!
//! Handles world initialization, registering actor classes, and setting up the initial world state.

use spacetimedb::ReducerContext;
use crate::actor::{ActorClass, ActorInfo, ActorLifecycleState};

/// Initializes the game world and populates initial actor class registry
pub fn initialize_world(ctx: &ReducerContext) {
    log::info!("Initializing actor world");
    
    // Register built-in actor classes
    register_default_actor_classes(ctx);
    
    // Create any persistent world actors if needed
    spawn_persistent_world_actors(ctx);
}

/// Registers the default/built-in actor classes that Unreal Engine provides
fn register_default_actor_classes(ctx: &ReducerContext) {
    log::info!("Registering default actor classes");
    
    // Default Unreal Engine actor classes
    let default_classes = [
        // Base actor class
        ActorClass {
            class_id: 1,
            class_name: "Actor".to_string(),
            class_path: "/Script/Engine.Actor".to_string(),
            replicates: true,
            always_relevant: false,
            requires_transform_updates: true,
        },
        
        // Pawn class
        ActorClass {
            class_id: 2,
            class_name: "Pawn".to_string(),
            class_path: "/Script/Engine.Pawn".to_string(),
            replicates: true,
            always_relevant: false,
            requires_transform_updates: true,
        },
        
        // Character class
        ActorClass {
            class_id: 3,
            class_name: "Character".to_string(),
            class_path: "/Script/Engine.Character".to_string(),
            replicates: true,
            always_relevant: false,
            requires_transform_updates: true,
        },
        
        // Player Controller
        ActorClass {
            class_id: 4,
            class_name: "PlayerController".to_string(),
            class_path: "/Script/Engine.PlayerController".to_string(),
            replicates: true,
            always_relevant: true, // Controllers are typically always relevant to their owning client
            requires_transform_updates: false,
        },
        
        // Game Mode
        ActorClass {
            class_id: 5,
            class_name: "GameMode".to_string(),
            class_path: "/Script/Engine.GameMode".to_string(),
            replicates: false, // GameMode doesn't replicate
            always_relevant: false,
            requires_transform_updates: false,
        },
        
        // Player State
        ActorClass {
            class_id: 6,
            class_name: "PlayerState".to_string(),
            class_path: "/Script/Engine.PlayerState".to_string(),
            replicates: true,
            always_relevant: false,
            requires_transform_updates: false,
        },
        
        // Game State
        ActorClass {
            class_id: 7,
            class_name: "GameState".to_string(),
            class_path: "/Script/Engine.GameState".to_string(),
            replicates: true,
            always_relevant: true, // GameState is typically relevant to all clients
            requires_transform_updates: false,
        },
    ];
    
    // Insert all default classes
    for class in default_classes.iter() {
        ctx.db.actor_class().insert(class.clone());
    }
    
    log::info!("Registered {} default actor classes", default_classes.len());
}

/// Creates any persistent actors that should exist in the world from the beginning
fn spawn_persistent_world_actors(ctx: &ReducerContext) {
    log::info!("Spawning persistent world actors");
    
    // Game State - Always exists in any Unreal game session
    let game_state_id = crate::actor::spawn::generate_actor_id();
    ctx.db.actor_info().insert(ActorInfo {
        actor_id: game_state_id,
        class_id: 7, // GameState class_id
        actor_name: "GameState".to_string(),
        owner_identity: None, // No owner - system actor
        state: ActorLifecycleState::Active,
        created_at: ctx.timestamp,
        hidden: false,
    });
    
    // Game Mode - Always exists on the server
    let game_mode_id = crate::actor::spawn::generate_actor_id();
    ctx.db.actor_info().insert(ActorInfo {
        actor_id: game_mode_id,
        class_id: 5, // GameMode class_id
        actor_name: "GameMode".to_string(),
        owner_identity: None, // No owner - system actor
        state: ActorLifecycleState::Active,
        created_at: ctx.timestamp,
        hidden: false,
    });
    
    log::info!("Created GameState (ID: {}) and GameMode (ID: {})", game_state_id, game_mode_id);
} 