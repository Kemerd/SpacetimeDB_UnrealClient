use serde::{Deserialize, Serialize};
use crate::object::ObjectId;

/// Transform data for an actor
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransformData {
    /// Location in world space
    pub location: [f32; 3],
    
    /// Rotation as quaternion [x, y, z, w]
    pub rotation: [f32; 4],
    
    /// Scale
    pub scale: [f32; 3],
}

/// Velocity data for an actor
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VelocityData {
    /// Linear velocity
    pub linear: [f32; 3],
    
    /// Angular velocity
    pub angular: [f32; 3],
}

/// Transform update sent to clients
#[derive(Debug, Serialize, Deserialize)]
pub struct TransformUpdate {
    /// Object ID this update is for
    pub object_id: ObjectId,
    
    /// The transform data
    pub transform: TransformData,
    
    /// The velocity data if applicable
    pub velocity: Option<VelocityData>,
    
    /// Sequence number for client prediction reconciliation
    pub sequence: Option<u32>,
}

impl TransformUpdate {
    /// Create a new transform update
    pub fn new(object_id: ObjectId, transform: TransformData, velocity: Option<VelocityData>) -> Self {
        Self {
            object_id,
            transform,
            velocity,
            sequence: None,
        }
    }
    
    /// Create a new transform update with sequence number
    pub fn with_sequence(object_id: ObjectId, transform: TransformData, velocity: Option<VelocityData>, sequence: u32) -> Self {
        Self {
            object_id,
            transform,
            velocity,
            sequence: Some(sequence),
        }
    }
    
    /// Set the sequence number
    pub fn set_sequence(&mut self, sequence: u32) {
        self.sequence = Some(sequence);
    }
} 