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
use log::{debug, error, info, warn};

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

    /// Indicates if this object has a temporary ID that needs to be remapped
    pub needs_id_remap: bool,
}

/// Global registry of client objects
static CLIENT_OBJECTS: Lazy<Mutex<HashMap<ObjectId, ClientObject>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Mapping of temporary client-generated IDs to server-authorized IDs
static TEMP_ID_MAPPING: Lazy<Mutex<HashMap<ObjectId, ObjectId>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Initialize the client-side object system
pub fn init() {
    info!("Initializing client-side object system");
    // Nothing to do here yet
}

/// Create an object on the client
pub fn create_object(class_name: &str, params: SpawnParams) -> Result<ObjectId, String> {
    // Generate a temporary ID (real ID will come from server)
    let temp_object_id = generate_temp_object_id();
    
    // Create a new client object with temporary ID
    let object = ClientObject {
        id: temp_object_id,
        class_name: class_name.to_string(),
        owner_id: params.owner_id,
        state: ObjectLifecycleState::Initializing,
        transform: params.transform,
        properties: HashMap::new(),
        needs_id_remap: true,  // Mark that this object needs ID remapping
    };
    
    // Register the object with temporary ID
    register_object(object);
    
    // Set initial properties in the local cache
    for (name, value_json) in &params.initial_properties {
        if let Ok(value) = crate::property::serialization::deserialize_property_value(value_json) {
            crate::property::cache_property_value(temp_object_id, name, value);
        }
    }
    
    // Request the server to create the object
    request_server_create_object(temp_object_id, class_name, &params)?;
    
    // Return the temporary object ID
    Ok(temp_object_id)
}

/// Request the server to create an object
fn request_server_create_object(temp_id: ObjectId, class_name: &str, params: &SpawnParams) -> Result<(), String> {
    // Check if connected to the server
    if !crate::net::is_connected() {
        return Err("Not connected to server".to_string());
    }
    
    // Extract transform components if any
    let (position, rotation, scale) = if let Some(transform) = &params.transform {
        (
            (transform.location.x, transform.location.y, transform.location.z),
            (transform.rotation.x, transform.rotation.y, transform.rotation.z, transform.rotation.w),
            (transform.scale.x, transform.scale.y, transform.scale.z)
        )
    } else {
        // Default transform values if none provided
        ((0.0, 0.0, 0.0), (0.0, 0.0, 0.0, 1.0), (1.0, 1.0, 1.0))
    };
    
    // Convert properties to (String, String) pairs for the server
    let initial_properties: Vec<(String, String)> = params.initial_properties.iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    
    // Format the properties as JSON
    let properties_json = serde_json::to_string(&initial_properties)
        .map_err(|e| format!("Failed to serialize properties: {}", e))?;
    
    // Get class ID from name
    let class_id = crate::class::get_class_id_by_name(class_name)
        .ok_or_else(|| format!("Unknown class: {}", class_name))?;
    
    // Call the appropriate server function to spawn the actor/object
    let args_json = format!(
        r#"{{"class_id": {}, "actor_name": "{}", "position": [{}, {}, {}], "rotation": [{}, {}, {}, {}], "scale": [{}, {}, {}], "initial_properties": {}}}"#,
        class_id, 
        class_name,  // Using class name as default actor name
        position.0, position.1, position.2,
        rotation.0, rotation.1, rotation.2, rotation.3,
        scale.0, scale.1, scale.2,
        properties_json
    );
    
    // Register a callback to handle the server's response with the real ID
    let temp_id_clone = temp_id;
    crate::rpc::register_callback("object_creation_response", move |_, response_json| {
        handle_object_creation_response(temp_id_clone, response_json)
    });
    
    // Call the server spawn function (actor or regular object based on class)
    if crate::class::is_actor_class(class_id) {
        crate::rpc::call_server_function(0, "spawn_actor", &args_json)?;
    } else {
        // For non-actor objects, use a different server function if available
        crate::rpc::call_server_function(0, "spawn_object", &args_json)?;
    }
    
    info!("Requested server to create object of class {} with temp ID {}", class_name, temp_id);
    Ok(())
}

/// Handle the server's response to an object creation request
fn handle_object_creation_response(temp_id: ObjectId, response_json: &str) -> Result<(), String> {
    // Parse the response to get the server-assigned ID
    let response: serde_json::Value = serde_json::from_str(response_json)
        .map_err(|e| format!("Failed to parse object creation response: {}", e))?;
    
    let server_id = response["object_id"].as_u64()
        .ok_or_else(|| "Missing object_id in server response".to_string())?;
    
    if server_id == 0 {
        // Server failed to create the object
        let error_msg = response["error"].as_str()
            .unwrap_or("Unknown error creating object on server")
            .to_string();
        
        // Clean up local temporary object
        let mut objects = CLIENT_OBJECTS.lock().unwrap();
        objects.remove(&temp_id);
        
        return Err(error_msg);
    }
    
    // Store the mapping from temporary ID to server ID
    {
        let mut id_mapping = TEMP_ID_MAPPING.lock().unwrap();
        id_mapping.insert(temp_id, server_id);
        
        debug!("Mapped temporary ID {} to server ID {}", temp_id, server_id);
    }
    
    // Update the object's state and ID if needed
    remap_object_id(temp_id, server_id)?;
    
    Ok(())
}

/// Remap a temporary client-generated ID to a server-authorized ID
pub fn remap_object_id(temp_id: ObjectId, server_id: ObjectId) -> Result<(), String> {
    if temp_id == server_id {
        // No remapping needed
        return Ok(());
    }
    
    let mut objects = CLIENT_OBJECTS.lock().unwrap();
    
    // Get the object with the temporary ID
    if let Some(mut object) = objects.remove(&temp_id) {
        // Update the ID and needs_id_remap flag
        object.id = server_id;
        object.needs_id_remap = false;
        object.state = ObjectLifecycleState::Active;
        
        // Insert the object with the new ID
        objects.insert(server_id, object.clone());
        
        // Update property cache to use the new ID
        crate::property::remap_property_cache(temp_id, server_id);
        
        info!("Remapped object {} to server ID {}", temp_id, server_id);
        
        // Notify callbacks about ID remapping
        drop(objects); // Release lock before callback
        crate::ffi::invoke_on_object_id_remapped(temp_id, server_id);
        
        Ok(())
    } else {
        Err(format!("Temporary object with ID {} not found for remapping", temp_id))
    }
}

/// Destroy an object on the client
pub fn destroy_object(object_id: ObjectId) -> Result<(), String> {
    // Check if object exists
    let object = {
        let objects = CLIENT_OBJECTS.lock().unwrap();
        match objects.get(&object_id) {
            Some(obj) => obj.clone(),
            None => return Err(format!("Object {} not found", object_id))
        }
    };
    
    // Mark as pending kill locally
    {
        let mut objects = CLIENT_OBJECTS.lock().unwrap();
        if let Some(obj) = objects.get_mut(&object_id) {
            obj.state = ObjectLifecycleState::PendingKill;
        }
    }
    
    // If this object has a temporary ID and hasn't been confirmed by the server,
    // we can just remove it locally without server interaction
    if object.needs_id_remap {
        info!("Destroying temporary object {} locally", object_id);
        let mut objects = CLIENT_OBJECTS.lock().unwrap();
        objects.remove(&object_id);
        return Ok(());
    }
    
    // Request the server to destroy the object
    request_server_destroy_object(object_id, object.class_name.as_str())?;
    
    Ok(())
}

/// Request the server to destroy an object
fn request_server_destroy_object(object_id: ObjectId, class_name: &str) -> Result<(), String> {
    // Check if connected to the server
    if !crate::net::is_connected() {
        return Err("Not connected to server".to_string());
    }
    
    // Get class ID from name
    let class_id = crate::class::get_class_id_by_name(class_name)
        .ok_or_else(|| format!("Unknown class: {}", class_name))?;
    
    // Choose the appropriate server function based on object type
    let function_name = if crate::class::is_actor_class(class_id) {
        "destroy_actor"
    } else {
        "destroy_object"
    };
    
    // Call the server function
    let args_json = format!(r#"{{"object_id": {}}}"#, object_id);
    crate::rpc::call_server_function(0, function_name, &args_json)?;
    
    info!("Requested server to destroy object {} of class {}", object_id, class_name);
    Ok(())
}

/// Handle server notification about an object being destroyed
pub fn handle_server_destroy_notification(object_id: ObjectId) {
    info!("Received server notification to destroy object {}", object_id);
    
    // Update local state
    let mut objects = CLIENT_OBJECTS.lock().unwrap();
    
    if let Some(mut object) = objects.get_mut(&object_id) {
        // Mark as destroyed
        object.state = ObjectLifecycleState::Destroyed;
    }
    
    // Remove from map
    objects.remove(&object_id);
    
    // Clean up property cache
    crate::property::clear_object_properties(object_id);
    
    // Notify FFI callbacks
    drop(objects); // Release lock before callback
    crate::ffi::invoke_on_object_destroyed(object_id);
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
    crate::property::cache_property_value(object_id, property_name, value.clone());
    
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
                    // Convert rotator to quaternion
                    // Proper implementation of Rotator to Quaternion conversion
                    let pitch = rotator.pitch * std::f32::consts::PI / 180.0;
                    let yaw = rotator.yaw * std::f32::consts::PI / 180.0;
                    let roll = rotator.roll * std::f32::consts::PI / 180.0;
                    
                    // Calculate components
                    let cy = (yaw * 0.5).cos();
                    let sy = (yaw * 0.5).sin();
                    let cp = (pitch * 0.5).cos();
                    let sp = (pitch * 0.5).sin();
                    let cr = (roll * 0.5).cos();
                    let sr = (roll * 0.5).sin();
                    
                    let quat = Quat {
                        x: cr * sp * cy - sr * cp * sy,
                        y: cr * cp * sy + sr * sp * cy,
                        z: sr * cp * cy - cr * sp * sy,
                        w: cr * cp * cy + sr * sp * sy,
                    };
                    
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
    
    // Ensure the ID is flagged as temporary by setting high-order bit
    // This ensures it won't collide with server IDs
    (timestamp ^ random) | (1u64 << 63)
} 