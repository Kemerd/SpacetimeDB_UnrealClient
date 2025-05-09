//! # Actor Types
//!
//! Shared actor-related types used by both client and server.
//! This module contains common definitions for actor data structures
//! and enums that are needed on both sides of the network connection.

use serde::{Serialize, Deserialize};
use crate::lifecycle::ActorLifecycleState;
use crate::types::{Vector3, Quat, Transform};
use crate::object::ObjectId;

/// Basic actor class information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorClassInfo {
    /// Unique identifier for the class
    pub class_id: u32,
    
    /// Name of the actor class
    pub class_name: String,
    
    /// Whether this actor class can be network-replicated
    pub replicates: bool,
    
    /// Whether actors of this class are relevant to all clients
    pub always_relevant: bool,
}

/// Basic actor instance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorInfo {
    /// Unique identifier for this actor instance
    pub actor_id: ObjectId,
    
    /// The class this actor belongs to
    pub class_id: u32,
    
    /// Actor instance name (if any)
    pub actor_name: String,
    
    /// Owner of this actor (if any)
    pub owner_client_id: Option<u64>,
    
    /// Current lifecycle state
    pub state: ActorLifecycleState,
    
    /// Whether this actor is hidden
    pub hidden: bool,
}

/// Compact representation of actor transform for network updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorTransform {
    /// Actor identifier
    pub actor_id: ObjectId,
    
    /// Position
    pub position: Vector3,
    
    /// Rotation
    pub rotation: Quat,
    
    /// Scale
    pub scale: Vector3,
}

impl From<ActorTransform> for Transform {
    fn from(t: ActorTransform) -> Self {
        Transform {
            location: t.position,
            rotation: t.rotation,
            scale: t.scale,
        }
    }
}

/// Actor spawn parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorSpawnParams {
    /// Class ID to spawn
    pub class_id: u32,
    
    /// Optional name for the actor
    pub actor_name: Option<String>,
    
    /// Initial transform
    pub initial_transform: Option<Transform>,
    
    /// Owner client ID (if any)
    pub owner_client_id: Option<u64>,
    
    /// Initial properties as JSON string
    pub initial_properties: Option<String>,
}

/// Actor movement update modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MovementMode {
    /// No movement
    None,
    
    /// Walking on a surface
    Walking,
    
    /// Falling under gravity
    Falling,
    
    /// Swimming in a fluid volume
    Swimming,
    
    /// Flying (no gravity)
    Flying,
    
    /// Custom movement mode
    Custom(u8),
} 