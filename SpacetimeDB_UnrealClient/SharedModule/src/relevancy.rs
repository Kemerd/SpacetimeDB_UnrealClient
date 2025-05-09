//! # Network Relevancy Types
//!
//! Shared types for network relevancy management. This system controls what objects
//! and actors are relevant to which clients, similar to Unreal Engine's native
//! relevancy system.

use serde::{Serialize, Deserialize};
use crate::object::ObjectId;

/// Represents a zone or area for relevancy purposes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelevancyZone {
    /// Unique zone identifier
    pub zone_id: u32,
    
    /// Zone name for debugging
    pub name: String,
    
    /// Whether zone is active
    pub active: bool,
}

/// Relevancy level for network objects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelevancyLevel {
    /// Always relevant to all clients
    AlwaysRelevant,
    
    /// Only relevant to owner
    OwnerOnly,
    
    /// Relevant based on distance
    DistanceBased,
    
    /// Relevant to clients in same zone
    SameZone,
    
    /// Custom relevancy logic
    Custom,
    
    /// Never relevant (server only)
    NeverRelevant,
}

/// Update frequency for network objects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateFrequency {
    /// Every tick
    High,
    
    /// Every other tick
    Medium,
    
    /// Every fourth tick
    Low,
    
    /// When explicitly requested
    OnDemand,
}

/// Network priority for objects
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum NetworkPriority {
    /// Critical updates that must go through
    Critical = 0,
    
    /// High priority updates (player controlled actors)
    High = 1,
    
    /// Normal priority (most gameplay actors)
    Normal = 2,
    
    /// Low priority (background actors)
    Low = 3,
}

/// Defines relevancy settings for an object or actor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelevancySettings {
    /// Object these settings apply to
    pub object_id: ObjectId,
    
    /// Relevancy level
    pub level: RelevancyLevel,
    
    /// Update frequency
    pub frequency: UpdateFrequency,
    
    /// Network priority
    pub priority: NetworkPriority,
    
    /// Maximum relevancy distance (if distance-based)
    pub max_distance: Option<f32>,
}

/// Represents zones an object or client is in
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneMembership {
    /// Object or client ID
    pub entity_id: u64,
    
    /// Is this a client or object
    pub is_client: bool,
    
    /// Zones this entity is in
    pub zone_ids: Vec<u32>,
} 