//! # Client-side Object System
//!
//! Handles client-side object management, including object creation, destruction,
//! and property management.

use stdb_shared::object::{ObjectId, SpawnParams};
use stdb_shared::lifecycle::ObjectLifecycleState;
use stdb_shared::property::PropertyValue;
use stdb_shared::types::*;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

// Re-export from shared for convenience
pub use stdb_shared::object::ObjectId;

/// Client-side object representation
#[derive(Debug, Clone)]
pub struct ClientObject {
    /// Unique object ID
    pub id: ObjectId,
    
    /// Class name
    pub class_name: String,
    
    /// Owner ID (if any)
    pub owner_id: Option<ObjectId>,
    
    /// Current lifecycle state
    pub state: ObjectLifecycleState,
    
    /// Object transform (for actors)
    pub transform: Option<Transform>,
    
    /// Properties
    pub properties: HashMap<String, PropertyValue>,
}

/// Global registry of client objects
static CLIENT_OBJECTS: Lazy<Mutex<HashMap<ObjectId, ClientObject>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Initialize the client-side object system
pub fn init() {
    println!("Initializing client-side object system");
    // Nothing to do here yet
}

/// Create an object on the client
pub fn create_object(class_name: &str, params: SpawnParams) -> Result<ObjectId, String> {
    // In a real implementation, we would request the server to create an object
    // For now, we'll create a placeholder
    
    // Generate a temporary ID (real ID will come from server)
    let object_id = generate_temp_object_id();
    
    // Create a new client object
    let object = ClientObject {
        id: object_id,
        class_name: class_name.to_string(),
        owner_id: params.owner_id,
        state: ObjectLifecycleState::Initializing,
        transform: params.transform,
        properties: HashMap::new(),
    };
    
    // Register the object
    register_object(object);
    
    // Set initial properties
    for (name, value_json) in &params.initial_properties {
        if let Ok(value) = crate::property::serialization::deserialize_property_value(value_json) {
            crate::property::cache_property_value(object_id, name, value);
        }
    }
    
    // Return the object ID
    Ok(object_id)
}

/// Destroy an object on the client
pub fn destroy_object(object_id: ObjectId) -> Result<(), String> {
    // In a real implementation, we would request the server to destroy the object
    // For now, we'll just update our local state
    
    let mut objects = CLIENT_OBJECTS.lock().unwrap();
    
    if let Some(mut object) = objects.get_mut(&object_id) {
        // Mark as pending kill
        object.state = ObjectLifecycleState::PendingKill;
        
        // In real implementation, we'd tell the server here
        
        // Return success
        Ok(())
    } else {
        Err(format!("Object {} not found", object_id))
    }
}

/// Get an object by ID
pub fn get_object(object_id: ObjectId) -> Option<ClientObject> {
    let objects = CLIENT_OBJECTS.lock().unwrap();
    objects.get(&object_id).cloned()
}

/// Get an object's class name
pub fn get_object_class(object_id: ObjectId) -> Option<String> {
    let objects = CLIENT_OBJECTS.lock().unwrap();
    objects.get(&object_id).map(|obj| obj.class_name.clone())
}

/// Update an object's property
pub fn update_object_property(
    object_id: ObjectId,
    property_name: &str,
    value: PropertyValue,
) -> Result<(), String> {
    // Cache the property value
    crate::property::cache_property_value(object_id, property_name, value);
    
    // Update the object if it exists
    let mut objects = CLIENT_OBJECTS.lock().unwrap();
    
    if let Some(object) = objects.get_mut(&object_id) {
        // Special handling for transform properties
        match property_name {
            "Location" => {
                if let PropertyValue::Vector(position) = &value {
                    if let Some(transform) = &mut object.transform {
                        transform.location = *position;
                    } else {
                        object.transform = Some(Transform {
                            location: *position,
                            rotation: Quat::identity(),
                            scale: Vector3::one(),
                        });
                    }
                }
            },
            "Rotation" => {
                if let PropertyValue::Rotator(rotator) = &value {
                    // Convert rotator to quaternion (simplified)
                    // In a real implementation, we'd use proper conversion
                    let quat = Quat::identity(); // Placeholder
                    
                    if let Some(transform) = &mut object.transform {
                        transform.rotation = quat;
                    } else {
                        object.transform = Some(Transform {
                            location: Vector3::zero(),
                            rotation: quat,
                            scale: Vector3::one(),
                        });
                    }
                }
                else if let PropertyValue::Quat(quat) = &value {
                    if let Some(transform) = &mut object.transform {
                        transform.rotation = *quat;
                    } else {
                        object.transform = Some(Transform {
                            location: Vector3::zero(),
                            rotation: *quat,
                            scale: Vector3::one(),
                        });
                    }
                }
            },
            "Scale" => {
                if let PropertyValue::Vector(scale) = &value {
                    if let Some(transform) = &mut object.transform {
                        transform.scale = *scale;
                    } else {
                        object.transform = Some(Transform {
                            location: Vector3::zero(),
                            rotation: Quat::identity(),
                            scale: *scale,
                        });
                    }
                }
            },
            _ => {
                // Regular property
                object.properties.insert(property_name.to_string(), value);
            }
        }
    }
    
    Ok(())
}

/// Register an object in the client registry
fn register_object(object: ClientObject) {
    let mut objects = CLIENT_OBJECTS.lock().unwrap();
    objects.insert(object.id, object);
}

/// Generate a temporary object ID
/// In a real implementation, object IDs would be assigned by the server
fn generate_temp_object_id() -> ObjectId {
    // Simple placeholder implementation that generates a "unique" ID
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    
    // Add a random component
    let random = (rand::random::<u32>() as u64) << 32;
    
    // Combine for a unique ID
    timestamp ^ random
} 