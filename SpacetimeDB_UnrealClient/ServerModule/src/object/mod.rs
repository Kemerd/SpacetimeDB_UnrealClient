//! # UObject Module
//! 
//! Provides the core UObject functionality which is the base for Actors and other replicable objects
//! in Unreal Engine. This handles the fundamental object model for our replication system.

use spacetimedb::{ReducerContext, Table, Identity};
use stdb_shared::property::{PropertyType, PropertyValue};
use stdb_shared::lifecycle::ObjectLifecycleState;

// Submodules
pub mod class;     // UClass definitions and registration
pub mod instance;  // Object instance management
pub mod field;     // Property field definitions

/// Unique identifier for UObjects in the SpacetimeDB system
pub type ObjectId = u64;

/// Represents a class type in Unreal's reflection system (UClass)
#[spacetimedb::table(name = object_class, public)]
pub struct ObjectClass {
    /// Unique identifier for the class
    #[primarykey]
    pub class_id: u32,
    
    /// Name of the class (e.g., "Actor", "Object", "Component")
    pub class_name: String,
    
    /// Path to the class in Unreal's asset system
    pub class_path: String,
    
    /// Parent class ID (0 for UObject which is the root)
    pub parent_class_id: u32,
    
    /// Whether this class can be network-replicated
    pub replicates: bool,
    
    /// Whether this class represents an Actor (vs a pure UObject)
    pub is_actor: bool,
    
    /// Whether this class represents a Component
    pub is_component: bool,
}

/// Core table for UObject instance data
#[spacetimedb::table(name = object_instance, public)]
pub struct ObjectInstance {
    /// Unique identifier for this object instance
    #[primarykey]
    pub object_id: ObjectId,
    
    /// The class this object belongs to
    pub class_id: u32,
    
    /// Object instance name (if any)
    pub object_name: String,
    
    /// Owner of this object (if any) - affects access control
    pub owner_identity: Option<Identity>,
    
    /// Outer object that contains this one (if any) - UObject hierarchy
    pub outer_object_id: Option<ObjectId>,
    
    /// When the object was created
    pub created_at: u64,
    
    /// Current object lifecycle state
    pub state: ObjectLifecycleState,
}

/// Object property values (for dynamic properties)
#[spacetimedb::table(name = object_property, public)]
pub struct ObjectProperty {
    /// Object this property belongs to
    #[primarykey]
    pub object_id: ObjectId,
    
    /// Property name 
    #[primarykey]
    pub property_name: String,
    
    /// Property value
    pub value: PropertyValue,
    
    /// Last update timestamp
    pub last_updated: u64,
    
    /// Whether this property replicates to clients
    pub replicated: bool,
}

/// Property field definitions on classes
#[spacetimedb::table(name = class_property, public)]
pub struct ClassProperty {
    /// Class that owns this property definition
    #[primarykey]
    pub class_id: u32,
    
    /// Property name
    #[primarykey]
    pub property_name: String,
    
    /// Property type information
    pub property_type: PropertyType,
    
    /// Whether the property replicates to clients by default
    pub replicated: bool,
    
    /// Whether the property is readonly for clients
    pub readonly: bool,
}

/// Object reference - for efficient tracking of graph relationships
#[spacetimedb::table(name = object_reference, public)]
pub struct ObjectReference {
    /// Source object making the reference
    #[primarykey]
    pub source_object_id: ObjectId,
    
    /// Property containing the reference
    #[primarykey]
    pub property_name: String,
    
    /// Target object being referenced
    pub target_object_id: ObjectId,
} 