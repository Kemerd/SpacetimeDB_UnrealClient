//! # UObject Class System
//!
//! Handles registration and management of UClass definitions, providing the 
//! reflection capabilities that are core to Unreal's object model.

use spacetimedb::ReducerContext;
use crate::object::{ObjectClass, ClassProperty};
use crate::property::PropertyType;

/// Initializes the UObject class system with standard Unreal Engine classes
pub fn initialize_object_classes(ctx: &ReducerContext) {
    log::info!("Initializing UObject class system");
    
    // Register core UObject classes
    register_core_classes(ctx);
    
    // Register property definitions for core classes
    register_core_properties(ctx);
}

/// Registers the fundamental classes from Unreal Engine
fn register_core_classes(ctx: &ReducerContext) {
    log::info!("Registering core UObject classes");
    
    // Core UObject classes with hierarchy
    let core_classes = [
        // UObject - the root of all objects
        ObjectClass {
            class_id: 1,
            class_name: "Object".to_string(),
            class_path: "/Script/CoreUObject.Object".to_string(),
            parent_class_id: 0, // No parent (root)
            replicates: false,  // Default UObject doesn't replicate
            is_actor: false,
            is_component: false,
        },
        
        // UActorComponent - base for all components
        ObjectClass {
            class_id: 2,
            class_name: "ActorComponent".to_string(), 
            class_path: "/Script/Engine.ActorComponent".to_string(),
            parent_class_id: 1, // UObject
            replicates: true,
            is_actor: false,
            is_component: true,
        },
        
        // USceneComponent - components with transforms
        ObjectClass {
            class_id: 3,
            class_name: "SceneComponent".to_string(),
            class_path: "/Script/Engine.SceneComponent".to_string(),
            parent_class_id: 2, // UActorComponent
            replicates: true,
            is_actor: false,
            is_component: true,
        },
        
        // UPrimitiveComponent - components that can render/collide
        ObjectClass {
            class_id: 4,
            class_name: "PrimitiveComponent".to_string(),
            class_path: "/Script/Engine.PrimitiveComponent".to_string(),
            parent_class_id: 3, // USceneComponent
            replicates: true,
            is_actor: false,
            is_component: true,
        },
        
        // UMeshComponent - base for mesh components
        ObjectClass {
            class_id: 5,
            class_name: "MeshComponent".to_string(),
            class_path: "/Script/Engine.MeshComponent".to_string(),
            parent_class_id: 4, // UPrimitiveComponent
            replicates: true,
            is_actor: false,
            is_component: true,
        },
        
        // UStaticMeshComponent - static mesh rendering
        ObjectClass {
            class_id: 6,
            class_name: "StaticMeshComponent".to_string(),
            class_path: "/Script/Engine.StaticMeshComponent".to_string(),
            parent_class_id: 5, // UMeshComponent
            replicates: true,
            is_actor: false,
            is_component: true,
        },
        
        // USkeletalMeshComponent - skeletal mesh rendering
        ObjectClass {
            class_id: 7,
            class_name: "SkeletalMeshComponent".to_string(),
            class_path: "/Script/Engine.SkeletalMeshComponent".to_string(),
            parent_class_id: 5, // UMeshComponent
            replicates: true,
            is_actor: false,
            is_component: true,
        },
        
        // AActor - base for all actors
        ObjectClass {
            class_id: 10,
            class_name: "Actor".to_string(),
            class_path: "/Script/Engine.Actor".to_string(),
            parent_class_id: 1, // UObject
            replicates: true,
            is_actor: true,
            is_component: false,
        },
        
        // APawn - controllable actors
        ObjectClass {
            class_id: 11,
            class_name: "Pawn".to_string(),
            class_path: "/Script/Engine.Pawn".to_string(),
            parent_class_id: 10, // AActor
            replicates: true,
            is_actor: true,
            is_component: false,
        },
        
        // ACharacter - humanoid pawns with character movement
        ObjectClass {
            class_id: 12,
            class_name: "Character".to_string(),
            class_path: "/Script/Engine.Character".to_string(),
            parent_class_id: 11, // APawn
            replicates: true,
            is_actor: true,
            is_component: false,
        },
        
        // AController - controls pawns
        ObjectClass {
            class_id: 13,
            class_name: "Controller".to_string(),
            class_path: "/Script/Engine.Controller".to_string(),
            parent_class_id: 10, // AActor
            replicates: true,
            is_actor: true,
            is_component: false,
        },
        
        // APlayerController - player-controlled actors
        ObjectClass {
            class_id: 14,
            class_name: "PlayerController".to_string(),
            class_path: "/Script/Engine.PlayerController".to_string(),
            parent_class_id: 13, // AController
            replicates: true,
            is_actor: true,
            is_component: false,
        },
        
        // AAIController - AI-controlled actors
        ObjectClass {
            class_id: 15,
            class_name: "AIController".to_string(),
            class_path: "/Script/Engine.AIController".to_string(),
            parent_class_id: 13, // AController
            replicates: true,
            is_actor: true,
            is_component: false,
        },
        
        // AGameMode - server-only game rules
        ObjectClass {
            class_id: 16,
            class_name: "GameMode".to_string(),
            class_path: "/Script/Engine.GameMode".to_string(),
            parent_class_id: 10, // AActor
            replicates: false, // GameMode doesn't replicate
            is_actor: true,
            is_component: false,
        },
        
        // AGameState - replicated game state
        ObjectClass {
            class_id: 17,
            class_name: "GameState".to_string(),
            class_path: "/Script/Engine.GameState".to_string(),
            parent_class_id: 10, // AActor
            replicates: true,
            is_actor: true, 
            is_component: false,
        },
        
        // APlayerState - per-player state
        ObjectClass {
            class_id: 18,
            class_name: "PlayerState".to_string(),
            class_path: "/Script/Engine.PlayerState".to_string(), 
            parent_class_id: 10, // AActor
            replicates: true,
            is_actor: true,
            is_component: false,
        },
        
        // UMovementComponent - handles movement
        ObjectClass {
            class_id: 20,
            class_name: "MovementComponent".to_string(),
            class_path: "/Script/Engine.MovementComponent".to_string(),
            parent_class_id: 2, // UActorComponent
            replicates: true,
            is_actor: false,
            is_component: true,
        },
        
        // UCharacterMovementComponent - character-specific movement
        ObjectClass {
            class_id: 21,
            class_name: "CharacterMovementComponent".to_string(),
            class_path: "/Script/Engine.CharacterMovementComponent".to_string(),
            parent_class_id: 20, // UMovementComponent
            replicates: true,
            is_actor: false,
            is_component: true,
        },
    ];
    
    // Insert all core classes
    for class in core_classes.iter() {
        ctx.db.object_class().insert(class.clone());
    }
    
    log::info!("Registered {} core UObject classes", core_classes.len());
}

/// Registers standard property definitions for core classes
fn register_core_properties(ctx: &ReducerContext) {
    log::info!("Registering core class properties");
    
    // Common properties for all actors
    let actor_class_id = 10; // AActor
    
    let actor_properties = [
        // Base transform properties
        ClassProperty {
            class_id: actor_class_id,
            property_name: "Location".to_string(),
            property_type: PropertyType::Vector,
            replicated: true,
            readonly: false,
        },
        ClassProperty {
            class_id: actor_class_id,
            property_name: "Rotation".to_string(),
            property_type: PropertyType::Rotator,
            replicated: true,
            readonly: false,
        },
        ClassProperty {
            class_id: actor_class_id,
            property_name: "Scale".to_string(),
            property_type: PropertyType::Vector,
            replicated: true,
            readonly: false,
        },
        ClassProperty {
            class_id: actor_class_id,
            property_name: "bHidden".to_string(),
            property_type: PropertyType::Bool,
            replicated: true,
            readonly: false,
        },
        ClassProperty {
            class_id: actor_class_id,
            property_name: "LifeSpan".to_string(),
            property_type: PropertyType::Float,
            replicated: true,
            readonly: false,
        },
    ];
    
    // Insert actor properties
    for prop in actor_properties.iter() {
        ctx.db.class_property().insert(prop.clone());
    }
    
    // Character properties
    let character_class_id = 12; // ACharacter
    
    let character_properties = [
        ClassProperty {
            class_id: character_class_id,
            property_name: "bIsCrouched".to_string(),
            property_type: PropertyType::Bool,
            replicated: true,
            readonly: false,
        },
        ClassProperty {
            class_id: character_class_id,
            property_name: "JumpMaxCount".to_string(),
            property_type: PropertyType::Int32,
            replicated: true,
            readonly: false,
        },
    ];
    
    // Insert character properties
    for prop in character_properties.iter() {
        ctx.db.class_property().insert(prop.clone());
    }
    
    // Player state properties
    let player_state_class_id = 18; // APlayerState
    
    let player_state_properties = [
        ClassProperty {
            class_id: player_state_class_id,
            property_name: "PlayerName".to_string(),
            property_type: PropertyType::String,
            replicated: true,
            readonly: false,
        },
        ClassProperty {
            class_id: player_state_class_id,
            property_name: "Score".to_string(),
            property_type: PropertyType::Float,
            replicated: true,
            readonly: false,
        },
    ];
    
    // Insert player state properties
    for prop in player_state_properties.iter() {
        ctx.db.class_property().insert(prop.clone());
    }
    
    log::info!("Registered core class properties");
}

/// Register a custom class (exposed to clients)
#[spacetimedb::reducer]
pub fn register_custom_class(
    ctx: &ReducerContext,
    class_name: String,
    class_path: String,
    parent_class_id: u32,
    replicates: bool,
    is_actor: bool,
    is_component: bool,
) -> u32 {
    // Verify caller has permission to register classes
    if !crate::connection::auth::is_admin(ctx) {
        log::warn!("Unauthorized client {:?} attempted to register class {}", ctx.sender, class_name);
        return 0;
    }
    
    // Verify parent class exists
    if parent_class_id > 0 {
        if ctx.db.object_class().filter_by_class_id(&parent_class_id).is_empty() {
            log::warn!("Attempted to register class with non-existent parent class ID: {}", parent_class_id);
            return 0;
        }
    }
    
    // Generate a new class ID
    // In a real implementation, you'd have a more robust ID allocation mechanism
    let class_id = generate_class_id(ctx);
    
    // Create new class
    ctx.db.object_class().insert(ObjectClass {
        class_id,
        class_name: class_name.clone(),
        class_path,
        parent_class_id,
        replicates,
        is_actor,
        is_component,
    });
    
    log::info!("Registered custom class: {} (ID: {})", class_name, class_id);
    
    // Return the assigned class ID
    class_id
}

/// Register a property on a class
#[spacetimedb::reducer]
pub fn register_class_property(
    ctx: &ReducerContext,
    class_id: u32,
    property_name: String,
    property_type: PropertyType,
    replicated: bool,
    readonly: bool,
) -> bool {
    // Verify caller has permission 
    if !crate::connection::auth::is_admin(ctx) {
        log::warn!("Unauthorized client {:?} attempted to register property", ctx.sender);
        return false;
    }
    
    // Verify class exists
    if ctx.db.object_class().filter_by_class_id(&class_id).is_empty() {
        log::warn!("Attempted to register property for non-existent class ID: {}", class_id);
        return false;
    }
    
    // Register the property
    ctx.db.class_property().insert(ClassProperty {
        class_id,
        property_name: property_name.clone(),
        property_type,
        replicated,
        readonly,
    });
    
    log::info!("Registered property '{}' on class {}", property_name, class_id);
    true
}

/// Generate a new unique class ID
fn generate_class_id(ctx: &ReducerContext) -> u32 {
    // Find the highest existing class ID and increment by 1
    let max_id = ctx.db.object_class()
        .iter()
        .map(|class| class.class_id)
        .max()
        .unwrap_or(0);
    
    // Start from 1000 to avoid collision with built-in classes
    std::cmp::max(max_id + 1, 1000)
} 