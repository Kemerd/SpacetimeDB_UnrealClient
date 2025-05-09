use serde::{Deserialize, Serialize};
use crate::object::{ObjectId, TransformData, VelocityData};

/// Sequence number for prediction
pub type SequenceNumber = u32;

/// Structure to track prediction state for an object
#[derive(Debug, Serialize, Deserialize)]
pub struct PredictionState {
    /// Object ID this prediction state belongs to
    pub object_id: ObjectId,
    
    /// Current sequence number for this object
    pub current_sequence: SequenceNumber,
    
    /// The last sequence number that was acknowledged by the server
    pub last_acked_sequence: SequenceNumber,
}

/// Transform update with prediction data
#[derive(Debug, Serialize, Deserialize)]
pub struct PredictedTransformUpdate {
    /// Object ID this update is for
    pub object_id: ObjectId,
    
    /// The sequence number for this update
    pub sequence: SequenceNumber,
    
    /// The transform data
    pub transform: TransformData,
    
    /// The velocity data if applicable
    pub velocity: Option<VelocityData>,
}

/// State of client-side prediction system
pub struct PredictionSystem {
    /// Objects that have active prediction
    prediction_states: std::collections::HashMap<ObjectId, PredictionState>,
}

impl PredictionSystem {
    /// Create a new prediction system
    pub fn new() -> Self {
        Self {
            prediction_states: std::collections::HashMap::new(),
        }
    }
    
    /// Register an object for prediction
    pub fn register_object(&mut self, object_id: ObjectId) {
        self.prediction_states.insert(object_id, PredictionState {
            object_id,
            current_sequence: 0,
            last_acked_sequence: 0,
        });
    }
    
    /// Unregister an object from prediction
    pub fn unregister_object(&mut self, object_id: ObjectId) {
        self.prediction_states.remove(&object_id);
    }
    
    /// Get the next sequence number for an object
    pub fn get_next_sequence(&mut self, object_id: ObjectId) -> Option<SequenceNumber> {
        if let Some(state) = self.prediction_states.get_mut(&object_id) {
            let seq = state.current_sequence;
            state.current_sequence = state.current_sequence.wrapping_add(1);
            Some(seq)
        } else {
            None
        }
    }
    
    /// Process a server acknowledgement
    pub fn process_ack(&mut self, object_id: ObjectId, sequence: SequenceNumber) {
        if let Some(state) = self.prediction_states.get_mut(&object_id) {
            state.last_acked_sequence = sequence;
        }
    }
    
    /// Check if an object has active prediction
    pub fn has_prediction(&self, object_id: ObjectId) -> bool {
        self.prediction_states.contains_key(&object_id)
    }
    
    /// Get the last acknowledged sequence for an object
    pub fn get_last_acked_sequence(&self, object_id: ObjectId) -> Option<SequenceNumber> {
        self.prediction_states.get(&object_id).map(|state| state.last_acked_sequence)
    }
}

// Default instance to be used
static mut PREDICTION_SYSTEM: Option<PredictionSystem> = None;

/// Initialize the prediction system
pub fn initialize() {
    unsafe {
        PREDICTION_SYSTEM = Some(PredictionSystem::new());
    }
}

/// Get the prediction system instance
pub fn get_prediction_system() -> Option<&'static mut PredictionSystem> {
    unsafe {
        PREDICTION_SYSTEM.as_mut()
    }
} 