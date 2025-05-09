//! # Shared Object System
//!
//! Common object types and IDs used across client and server.

use serde::{Serialize, Deserialize};
use crate::types::*;
use crate::lifecycle::ObjectLifecycleState;
use std::collections::HashMap;

/// Unique identifier for UObjects in the SpacetimeDB system
pub type ObjectId = u64;

/// Unique identifier for class definitions
pub type ClassId = u32;

/// Object spawn parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnParams {
    /// Class name of the object to spawn
    pub class_name: String,
    
    /// Initial transform (only used for actors)
    pub transform: Option<Transform>,
    
    /// Initial owner
    pub owner_id: Option<ObjectId>,
    
    /// Whether the object can replicate
    pub replicates: bool,
    
    /// Whether this is system-spawned (not user-created)
    pub is_system: bool,
    
    /// Initial property values
    pub initial_properties: HashMap<String, String>,
}

impl Default for SpawnParams {
    fn default() -> Self {
        Self {
            class_name: String::new(),
            transform: Some(Transform::identity()),
            owner_id: None,
            replicates: true,
            is_system: false,
            initial_properties: HashMap::new(),
        }
    }
}

/// Object description used in replication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectDescription {
    /// Object ID
    pub object_id: ObjectId,
    
    /// Class name
    pub class_name: String,
    
    /// Owner ID (if any)
    pub owner_id: Option<ObjectId>,
    
    /// Current lifecycle state
    pub state: ObjectLifecycleState,
    
    /// Whether object replicates
    pub replicates: bool,
    
    /// Transform (for actors)
    pub transform: Option<Transform>,
    
    /// Other properties as JSON strings
    pub properties: HashMap<String, String>,
} 