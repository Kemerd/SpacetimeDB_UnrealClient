//! # Actor Module
//! 
//! Handles actor lifecycle (spawning, destruction, registration) and core actor data structures.
//! This module provides the foundation for replicating Unreal Engine actors in SpacetimeDB.

use spacetimedb::{ReducerContext, Table, Identity};
use crate::property::{PropertyType, PropertyValue};

// Submodules
pub mod init;       // Actor world initialization
pub mod spawn;      // Actor spawning and creation
pub mod lifecycle;  // Actor lifecycle management
pub mod transform;  // Actor transform handling (position, rotation, scale)

/// Unique identifier for actors in the SpacetimeDB system
pub type ActorId = u64;

/// Represents a class of Unreal actors (equivalent to UClass)
#[spacetimedb::table(name = actor_class, public)]
pub struct ActorClass {
    /// Unique identifier for the class
    #[primarykey]
    pub class_id: u32,
    
    /// Name of the actor class (e.g., "Character", "PlayerController")
    pub class_name: String,
    
    /// Path to the actor class in Unreal's asset system
    pub class_path: String,
    
    /// Whether this actor class can be network-replicated
    pub replicates: bool,
    
    /// Whether actor of this class are relevant to all clients (static relevancy)
    pub always_relevant: bool,
    
    /// Whether this actor type needs full transform updates
    pub requires_transform_updates: bool,
}

/// Core table for actor instance data
#[spacetimedb::table(name = actor_info, public)]
pub struct ActorInfo {
    /// Unique identifier for this actor instance
    #[primarykey]
    pub actor_id: ActorId,
    
    /// The class this actor belongs to
    pub class_id: u32,
    
    /// Actor instance name (if any)
    pub actor_name: String,
    
    /// Owner of this actor (if any)
    pub owner_identity: Option<Identity>,
    
    /// Current lifecycle state
    pub state: ActorLifecycleState,
    
    /// When the actor was created
    pub created_at: u64,
    
    /// Whether this actor is hidden
    pub hidden: bool,
}

/// Represents the actor's position, rotation, and scale
#[spacetimedb::table(name = actor_transform, public)]
pub struct ActorTransform {
    /// Actor this transform belongs to
    #[primarykey]
    pub actor_id: ActorId,
    
    // Position
    pub pos_x: f32,
    pub pos_y: f32,
    pub pos_z: f32,
    
    // Rotation (quaternion)
    pub rot_x: f32,
    pub rot_y: f32,
    pub rot_z: f32,
    pub rot_w: f32,
    
    // Scale
    pub scale_x: f32,
    pub scale_y: f32,
    pub scale_z: f32,
}

/// The current state of an actor in its lifecycle
#[derive(spacetimedb::Serialize, spacetimedb::Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ActorLifecycleState {
    /// Actor is being created but not yet fully initialized
    Spawning,
    
    /// Actor is active in the world
    Active,
    
    /// Actor is being destroyed
    PendingDestroy,
    
    /// Actor has been destroyed but is kept in the database for delayed cleanup
    Destroyed,
}

/// Actor property values (for dynamic properties not in schema)
#[spacetimedb::table(name = actor_property, public)]
pub struct ActorProperty {
    /// Actor this property belongs to
    #[primarykey]
    pub actor_id: ActorId,
    
    /// Property name
    #[primarykey]
    pub property_name: String,
    
    /// Property value
    pub value: PropertyValue,
    
    /// Last update timestamp
    pub last_updated: u64,
}

/// Actor component instances
#[spacetimedb::table(name = actor_component, public)]
pub struct ActorComponent {
    /// Unique ID of this component instance
    #[primarykey]
    pub component_id: u64,
    
    /// Actor that owns this component
    pub owner_actor_id: ActorId,
    
    /// Component class type
    pub component_class_id: u32,
    
    /// Component instance name
    pub component_name: String,
    
    /// Whether this component is active
    pub is_active: bool,
} 