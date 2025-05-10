//! # Client-side Object System
//!
//! Handles client-side object management, including object creation, destruction,
//! and property management. This module handles both actor and non-actor objects through
//! a unified object representation.

use stdb_shared::object::ObjectId;
use stdb_shared::object::SpawnParams;
use stdb_shared::property::{PropertyType, PropertyValue};
use stdb_shared::types::*;
use stdb_shared::lifecycle::ObjectLifecycleState;
use crate::property;
use crate::net;
use crate::class::{get_class, has_class, get_parent_class};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use once_cell::sync::Lazy;
use log::{debug, error, info, warn};
use serde_json::{Value, json};

/// Client-side object representation
/// Handles both regular objects and actors in a unified structure
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
    
    /// Object transform (always present for actors, optional for non-actors)
    pub transform: Option<Transform>,
    
    /// Properties
    pub properties: HashMap<String, PropertyValue>,

    /// Indicates if this object has a temporary ID that needs to be remapped
    pub needs_id_remap: bool,
    
    /// Indicates if this object is an actor
    pub is_actor: bool,
    
    /// Whether this object replicates
    pub replicates: bool,
    
    /// Components attached to this object (only used if is_actor is true)
    pub components: Vec<ObjectId>,
}

impl ClientObject {
    /// Returns a reference to this object's transform, creating a default one if none exists
    pub fn get_transform(&mut self) -> &mut Transform {
        if self.transform.is_none() {
            self.transform = Some(Transform::identity());
        }
        self.transform.as_mut().unwrap()
    }
    
    /// Check if this object is an actor
    pub fn is_actor(&self) -> bool {
        self.is_actor
    }
    
    /// Get all components attached to this object (if it's an actor)
    pub fn get_components(&self) -> &Vec<ObjectId> {
        &self.components
    }
    
    /// Get all components as ClientObject instances (if it's an actor)
    pub fn get_component_objects(&self) -> Vec<ClientObject> {
        if !self.is_actor {
            return Vec::new();
        }
        
        let objects = CLIENT_OBJECTS.lock().unwrap();
        self.components.iter()
            .filter_map(|&component_id| objects.get(&component_id).cloned())
            .collect()
    }
    
    /// Get a specific component by class name (returns the first matching component)
    pub fn get_component_by_class(&self, class_name: &str) -> Option<ObjectId> {
        if !self.is_actor {
            return None;
        }
        
        let objects = CLIENT_OBJECTS.lock().unwrap();
        for &component_id in &self.components {
            if let Some(component) = objects.get(&component_id) {
                if component.class_name == class_name {
                    return Some(component_id);
                }
            }
        }
        
        None
    }
    
    /// Get a specific component object by class name (returns the first matching component)
    pub fn get_component_object_by_class(&self, class_name: &str) -> Option<ClientObject> {
        self.get_component_by_class(class_name).and_then(|id| {
            let objects = CLIENT_OBJECTS.lock().unwrap();
            objects.get(&id).cloned()
        })
    }
    
    /// Check if this object is a component
    pub fn is_component(&self) -> bool {
        let owners = COMPONENT_OWNERS.lock().unwrap();
        owners.contains_key(&self.id)
    }
    
    /// Get the owner of this component (if it is a component)
    pub fn get_owner_id(&self) -> Option<ObjectId> {
        if self.is_component() {
            let owners = COMPONENT_OWNERS.lock().unwrap();
            owners.get(&self.id).copied()
        } else {
            None
        }
    }
    
    /// Get the owner object of this component (if it is a component)
    pub fn get_owner(&self) -> Option<ClientObject> {
        self.get_owner_id().and_then(|owner_id| {
            let objects = CLIENT_OBJECTS.lock().unwrap();
            objects.get(&owner_id).cloned()
        })
    }
    
    /// Add a component to this object (if it's an actor)
    pub fn add_component(&mut self, component_id: ObjectId) -> Result<(), String> {
        if !self.is_actor {
            return Err(format!("Cannot add component to non-actor object {}", self.id));
        }
        
        if !self.components.contains(&component_id) {
            // Add component to this actor's component list
            self.components.push(component_id);
            
            // Register the component's owner
            let mut owners = COMPONENT_OWNERS.lock().unwrap();
            owners.insert(component_id, self.id);
            
            // Update the component's owner_id
            let mut objects = CLIENT_OBJECTS.lock().unwrap();
            if let Some(component) = objects.get_mut(&component_id) {
                component.owner_id = Some(self.id);
            }
            
            Ok(())
        } else {
            Err(format!("Component {} already attached to actor {}", component_id, self.id))
        }
    }
    
    /// Remove a component from this object (if it's an actor)
    pub fn remove_component(&mut self, component_id: ObjectId) -> Result<(), String> {
        if !self.is_actor {
            return Err(format!("Cannot remove component from non-actor object {}", self.id));
        }
        
        if self.components.contains(&component_id) {
            // Remove component from this actor's component list
            self.components.retain(|&id| id != component_id);
            
            // Unregister the component's owner
            let mut owners = COMPONENT_OWNERS.lock().unwrap();
            owners.remove(&component_id);
            
            // Update the component's owner_id
            let mut objects = CLIENT_OBJECTS.lock().unwrap();
            if let Some(component) = objects.get_mut(&component_id) {
                component.owner_id = None;
            }
        }
        
        Ok(())
    }
    
    /// Get a property value from this object
    pub fn get_property(&self, property_name: &str) -> Option<PropertyValue> {
        // Special handling for transform properties
        match property_name {
            "Location" => {
                self.transform.as_ref().map(|t| PropertyValue::Vector(t.location))
            },
            "Rotation" => {
                self.transform.as_ref().map(|t| PropertyValue::Quat(t.rotation))
            },
            "Scale" => {
                self.transform.as_ref().map(|t| PropertyValue::Vector(t.scale))
            },
            _ => {
                // Regular property - check the object's property map
                self.properties.get(property_name).cloned()
            }
        }
    }
    
    /// Set a property value on this object
    pub fn set_property(&mut self, property_name: &str, value: PropertyValue) -> Result<(), String> {
        // Special handling for transform properties
        match property_name {
            "Location" => {
                if let PropertyValue::Vector(location) = value {
                    self.get_transform().location = location;
                } else {
                    return Err(format!("Invalid value type for Location property"));
                }
            },
            "Rotation" => {
                match value {
                    PropertyValue::Quat(rotation) => {
                        self.get_transform().rotation = rotation;
                    },
                    PropertyValue::Rotator(rotator) => {
                        // Convert Rotator to Quaternion
                        // This is a simplified conversion - in a real implementation, 
                        // we'd use proper conversion logic
                        warn!("Using placeholder Rotator to Quat conversion");
                        self.get_transform().rotation = Quat::identity();
                    },
                    _ => return Err(format!("Invalid value type for Rotation property")),
                }
            },
            "Scale" => {
                if let PropertyValue::Vector(scale) = value {
                    self.get_transform().scale = scale;
                } else {
                    return Err(format!("Invalid value type for Scale property"));
                }
            },
            _ => {
                // Regular property - store in the property map
                self.properties.insert(property_name.to_string(), value);
            }
        }
        
        Ok(())
    }
}

/// Global registry of client objects (both actors and non-actors)
static CLIENT_OBJECTS: Lazy<Mutex<HashMap<ObjectId, ClientObject>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Mapping of component owners - tracks which actor owns which component
static COMPONENT_OWNERS: Lazy<Mutex<HashMap<ObjectId, ObjectId>>> =
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
    
    // Check if this class is an actor class
    let is_actor = crate::class::is_actor_class(
        crate::class::get_class_id_by_name(class_name).unwrap_or(0)
    );
    
    // Pre-cache any initial properties in the staging cache
    for (name, value_json) in &params.initial_properties {
        if let Ok(value) = crate::property::serialization::deserialize_property_value(value_json) {
            crate::property::cache_property_value(temp_object_id, name, value);
        }
    }
    
    // Transfer properties from staging cache to the object
    let properties = crate::property::transfer_cached_properties_to_object(temp_object_id);
    
    // Create a new client object with temporary ID
    let object = ClientObject {
        id: temp_object_id,
        class_name: class_name.to_string(),
        owner_id: params.owner_id,
        state: ObjectLifecycleState::Initializing,
        transform: params.transform,
        properties,
        needs_id_remap: true,  // Mark that this object needs ID remapping
        is_actor,
        replicates: params.replicates,
        components: Vec::new(),
    };
    
    // Register the object with temporary ID
    register_object_internal(object);
    
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

/// Remap an object's ID from a temporary client-assigned ID to a server-authoritative ID
pub fn remap_object_id(temp_id: ObjectId, server_id: ObjectId) -> Result<(), String> {
    let mut objects = CLIENT_OBJECTS.lock().unwrap();
    
    if let Some(mut object) = objects.remove(&temp_id) {
        // Update the object ID
        object.id = server_id;
        object.needs_id_remap = false;
        
        // Transfer any cached properties for this object
        let cached_properties = crate::property::transfer_cached_properties_to_object(server_id);
        
        // Merge them with existing properties (server properties take precedence)
        for (key, value) in cached_properties {
            object.properties.insert(key, value);
        }
        
        // Re-insert with new ID
        objects.insert(server_id, object);
        
        debug!("Remapped object from temporary ID {} to server ID {}", temp_id, server_id);
        Ok(())
    } else {
        Err(format!("Failed to remap object: temp ID {} not found", temp_id))
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
    request_server_destroy_object(object_id, &object.class_name)?;
    
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
    // First clean up any component ownership records
    {
        let mut owners = COMPONENT_OWNERS.lock().unwrap();
        owners.remove(&object_id); // If it's a component, remove its owner record
        
        // If it's an actor with components, remove all component owner records
        let mut objects = CLIENT_OBJECTS.lock().unwrap();
        if let Some(actor) = objects.get(&object_id) {
            if actor.is_actor {
                for &component_id in &actor.components {
                    owners.remove(&component_id);
                }
            }
        }
    }
    
    // Now proceed with existing destruction logic
    let mut objects = CLIENT_OBJECTS.lock().unwrap();
    objects.remove(&object_id);
    
    debug!("Removed object {} from client registry (server notification)", object_id);
    
    // Notify FFI layer about the destruction
    let callbacks = crate::ffi::CALLBACKS.lock().unwrap();
    crate::ffi::invoke_on_object_destroyed(callbacks.on_object_destroyed, object_id);
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
                // Regular property - update in the object's property map
                object.properties.insert(property_name.to_string(), value.clone());
            }
        }
        
        // Since this is an authoritative update, make sure the staging cache is also updated
        // for any code that might still be using it
        crate::property::cache_property_value(object_id, property_name, value);
        
        Ok(())
    } else {
        // If the object doesn't exist yet, store the property in the staging cache
        // This happens when properties arrive from the server before the object itself
        crate::property::cache_property_value(object_id, property_name, value);
        warn!("Object {} not found for property update '{}', storing in staging cache", object_id, property_name);
        Ok(())
    }
}

/// Register an object in the client registry
fn register_object_internal(object: ClientObject) {
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

/// Check if an object is an actor
pub fn is_actor(object_id: ObjectId) -> bool {
    let objects = CLIENT_OBJECTS.lock().unwrap();
    if let Some(obj) = objects.get(&object_id) {
        obj.is_actor
    } else {
        false
    }
}

/// Update an object's transform
pub fn update_transform(
    object_id: ObjectId,
    location: Option<Vector3>,
    rotation: Option<Quat>,
    scale: Option<Vector3>,
) -> Result<(), String> {
    let mut objects = CLIENT_OBJECTS.lock().unwrap();
    
    if let Some(object) = objects.get_mut(&object_id) {
        {
            // Ensure there's a transform to update
            let transform = object.get_transform();
            
            // Update location if provided
            if let Some(loc) = location {
                transform.location = loc;
            }
            
            // Update rotation if provided
            if let Some(rot) = rotation {
                transform.rotation = rot;
            }
            
            // Update scale if provided
            if let Some(scl) = scale {
                transform.scale = scl;
            }
        }
        
        // Now update properties
        if let Some(loc) = location {
            // Update the location property
            let loc_value = PropertyValue::Vector(loc);
            object.properties.insert("Location".to_string(), loc_value.clone());
            
            // Also update the staging cache for consistency
            crate::property::cache_property_value(object_id, "Location", loc_value);
        }
        
        if let Some(rot) = rotation {
            // Update the rotation property
            let rot_value = PropertyValue::Quat(rot);
            object.properties.insert("Rotation".to_string(), rot_value.clone());
            
            // Also update the staging cache for consistency
            crate::property::cache_property_value(object_id, "Rotation", rot_value);
        }
        
        if let Some(scl) = scale {
            // Update the scale property
            let scale_value = PropertyValue::Vector(scl);
            object.properties.insert("Scale".to_string(), scale_value.clone());
            
            // Also update the staging cache for consistency
            crate::property::cache_property_value(object_id, "Scale", scale_value);
        }
        
        Ok(())
    } else {
        Err(format!("Object {} not found", object_id))
    }
}

/// Update an object's lifecycle state
pub fn update_lifecycle_state(
    object_id: ObjectId,
    state: ObjectLifecycleState,
) -> Result<(), String> {
    let mut objects = CLIENT_OBJECTS.lock().unwrap();
    
    if let Some(object) = objects.get_mut(&object_id) {
        object.state = state;
        
        // If destroyed, remove from registry
        if state == ObjectLifecycleState::Destroyed {
            drop(objects); // Release lock before calling destroy_object
            destroy_object(object_id)?;
        }
        
        Ok(())
    } else {
        Err(format!("Object {} not found", object_id))
    }
}

/// Get a component by ID, ensuring it's actually a component
pub fn get_component(component_id: ObjectId) -> Option<ClientObject> {
    let owners = COMPONENT_OWNERS.lock().unwrap();
    if !owners.contains_key(&component_id) {
        return None; // Not a component
    }
    
    get_object(component_id)
}

/// Get all components of a specific actor
pub fn get_components(actor_id: ObjectId) -> Result<Vec<ObjectId>, String> {
    let objects = CLIENT_OBJECTS.lock().unwrap();
    
    if let Some(actor) = objects.get(&actor_id) {
        if !actor.is_actor {
            return Err(format!("Object {} is not an actor", actor_id));
        }
        Ok(actor.components.clone())
    } else {
        Err(format!("Actor {} not found", actor_id))
    }
}

/// Get all component objects of a specific actor
pub fn get_component_objects(actor_id: ObjectId) -> Result<Vec<ClientObject>, String> {
    let objects = CLIENT_OBJECTS.lock().unwrap();
    
    if let Some(actor) = objects.get(&actor_id) {
        if !actor.is_actor {
            return Err(format!("Object {} is not an actor", actor_id));
        }
        
        Ok(actor.components.iter()
            .filter_map(|&component_id| objects.get(&component_id).cloned())
            .collect())
    } else {
        Err(format!("Actor {} not found", actor_id))
    }
}

/// Get a component by class name
pub fn get_component_by_class(actor_id: ObjectId, class_name: &str) -> Result<Option<ClientObject>, String> {
    let objects = CLIENT_OBJECTS.lock().unwrap();
    
    if let Some(actor) = objects.get(&actor_id) {
        if !actor.is_actor {
            return Err(format!("Object {} is not an actor", actor_id));
        }
        
        for &component_id in &actor.components {
            if let Some(component) = objects.get(&component_id) {
                if component.class_name == class_name {
                    return Ok(Some(component.clone()));
                }
            }
        }
        
        Ok(None) // No component with that class name found
    } else {
        Err(format!("Actor {} not found", actor_id))
    }
}

/// Get the owner of a component
pub fn get_component_owner(component_id: ObjectId) -> Option<ObjectId> {
    let owners = COMPONENT_OWNERS.lock().unwrap();
    owners.get(&component_id).copied()
}

/// Check if an object is a component
pub fn is_component(object_id: ObjectId) -> bool {
    let owners = COMPONENT_OWNERS.lock().unwrap();
    owners.contains_key(&object_id)
}

/// Add a component to an actor
pub fn add_component(
    actor_id: ObjectId,
    component_id: ObjectId,
) -> Result<(), String> {
    // First check if the component exists
    {
        let objects = CLIENT_OBJECTS.lock().unwrap();
        if !objects.contains_key(&component_id) {
            return Err(format!("Component {} does not exist", component_id));
        }
    }
    
    // Now get the actor and add the component
    let mut objects = CLIENT_OBJECTS.lock().unwrap();
    
    if let Some(actor) = objects.get_mut(&actor_id) {
        actor.add_component(component_id)
    } else {
        Err(format!("Actor {} not found", actor_id))
    }
}

/// Create a new component and attach it to an actor
pub fn create_and_attach_component(
    actor_id: ObjectId,
    component_class: &str,
) -> Result<ObjectId, String> {
    // First check if actor exists
    {
        let objects = CLIENT_OBJECTS.lock().unwrap();
        if !objects.contains_key(&actor_id) || !objects.get(&actor_id).unwrap().is_actor {
            return Err(format!("Actor {} not found or not an actor", actor_id));
        }
    }
    
    // Create the component object
    let component_params = SpawnParams {
        class_name: component_class.to_string(),
        transform: None, // Components typically inherit transform from owner
        owner_id: Some(actor_id),
        replicates: true, // Components typically replicate with their owner
        initial_properties: HashMap::new(),
        is_system: false,
    };
    
    // Create the component
    let component_id = create_object(component_class, component_params)?;
    
    // Attach it to the actor
    add_component(actor_id, component_id)?;
    
    Ok(component_id)
}

/// Get a property from a component
pub fn get_component_property(
    actor_id: ObjectId,
    component_class: &str,
    property_name: &str,
) -> Result<Option<PropertyValue>, String> {
    // Find the component by class name
    let component = get_component_by_class(actor_id, component_class)?;
    
    // If component exists, get the property
    if let Some(component) = component {
        Ok(component.get_property(property_name))
    } else {
        Ok(None) // Component not found, but not an error
    }
}

/// Set a property on a component
pub fn set_component_property(
    actor_id: ObjectId,
    component_class: &str,
    property_name: &str,
    value: PropertyValue,
) -> Result<(), String> {
    // Find the component by class name
    let component_result = get_component_by_class(actor_id, component_class)?;
    
    if let Some(component) = component_result {
        // Update the component property
        update_object_property(component.id, property_name, value)
    } else {
        Err(format!("Component of class {} not found on actor {}", component_class, actor_id))
    }
}

/// Get all objects in the world
pub fn get_all_objects() -> Vec<ClientObject> {
    let objects = CLIENT_OBJECTS.lock().unwrap();
    objects.values().cloned().collect()
}

/// Get all actors in the world
pub fn get_all_actors() -> Vec<ClientObject> {
    let objects = CLIENT_OBJECTS.lock().unwrap();
    objects.values()
        .filter(|obj| obj.is_actor)
        .cloned()
        .collect()
}

/// Create an actor on the client (convenience wrapper around create_object)
pub fn create_actor(
    class_name: &str,
    transform: Transform,
    owner_id: Option<ObjectId>,
    replicates: bool,
) -> Result<ObjectId, String> {
    let mut params = SpawnParams::default();
    params.class_name = class_name.to_string();
    params.transform = Some(transform);
    params.owner_id = owner_id;
    params.replicates = replicates;
    
    create_object(class_name, params)
}

/// Get a property value from an object
pub fn get_object_property(
    object_id: ObjectId,
    property_name: &str,
) -> Option<PropertyValue> {
    let objects = CLIENT_OBJECTS.lock().unwrap();
    
    if let Some(object) = objects.get(&object_id) {
        // Special handling for transform properties
        match property_name {
            "Location" => {
                object.transform.as_ref().map(|t| PropertyValue::Vector(t.location))
            },
            "Rotation" => {
                object.transform.as_ref().map(|t| PropertyValue::Quat(t.rotation))
            },
            "Scale" => {
                object.transform.as_ref().map(|t| PropertyValue::Vector(t.scale))
            },
            _ => {
                // Regular property - check the object's property map
                object.properties.get(property_name).cloned()
            }
        }
    } else {
        // If the object doesn't exist in the registry, check the staging cache
        // This happens during object creation or when properties arrive before objects
        crate::property::get_cached_property_value(object_id, property_name)
    }
}

/// Remove a component from an actor
pub fn remove_component(
    actor_id: ObjectId,
    component_id: ObjectId,
) -> Result<(), String> {
    let mut objects = CLIENT_OBJECTS.lock().unwrap();
    
    if let Some(actor) = objects.get_mut(&actor_id) {
        actor.remove_component(component_id)
    } else {
        Err(format!("Actor {} not found", actor_id))
    }
}

/// Register an object on the client with the given class name and parameters
pub fn register_object(class_name: &str, params: &serde_json::Value) -> Result<ObjectId, String> {
    // Generate a temporary ID
    let object_id = generate_temp_object_id();
    
    // Check if this class is an actor class
    let is_actor = crate::class::is_actor_class(
        crate::class::get_class_id_by_name(class_name).unwrap_or(0)
    );
    
    // Extract initial properties from the params
    let mut initial_properties = HashMap::new();
    if let Some(props) = params.get("properties").and_then(|p| p.as_object()) {
        for (key, value) in props {
            if let Ok(value_str) = serde_json::to_string(value) {
                initial_properties.insert(key.clone(), value_str);
            }
        }
    }
    
    // Extract transform if provided
    let transform = if let Some(transform_data) = params.get("transform") {
        let location = transform_data.get("location").and_then(|l| {
            if let (Some(x), Some(y), Some(z)) = (
                l.get("x").and_then(|x| x.as_f64()),
                l.get("y").and_then(|y| y.as_f64()),
                l.get("z").and_then(|z| z.as_f64())
            ) {
                Some(Vector3 { x: x as f32, y: y as f32, z: z as f32 })
            } else {
                None
            }
        }).unwrap_or(Vector3::zero());
        
        let rotation = transform_data.get("rotation").and_then(|r| {
            if let (Some(x), Some(y), Some(z), Some(w)) = (
                r.get("x").and_then(|x| x.as_f64()),
                r.get("y").and_then(|y| y.as_f64()),
                r.get("z").and_then(|z| z.as_f64()),
                r.get("w").and_then(|w| w.as_f64())
            ) {
                Some(Quat { x: x as f32, y: y as f32, z: z as f32, w: w as f32 })
            } else {
                None
            }
        }).unwrap_or(Quat::identity());
        
        let scale = transform_data.get("scale").and_then(|s| {
            if let (Some(x), Some(y), Some(z)) = (
                s.get("x").and_then(|x| x.as_f64()),
                s.get("y").and_then(|y| y.as_f64()),
                s.get("z").and_then(|z| z.as_f64())
            ) {
                Some(Vector3 { x: x as f32, y: y as f32, z: z as f32 })
            } else {
                None
            }
        }).unwrap_or(Vector3::one());
        
        Some(Transform { location, rotation, scale })
    } else {
        None
    };
    
    // Extract owner_id if provided
    let owner_id = params.get("owner_id").and_then(|o| o.as_u64());
    
    // Extract replicates flag
    let replicates = params.get("replicates").and_then(|r| r.as_bool()).unwrap_or(true);
    
    // Pre-cache any initial properties in the staging cache
    for (name, value_json) in &initial_properties {
        if let Ok(value) = crate::property::serialization::deserialize_property_value(value_json) {
            crate::property::cache_property_value(object_id, name, value);
        }
    }
    
    // Transfer properties from staging cache to the object
    let properties = crate::property::transfer_cached_properties_to_object(object_id);
    
    // Create the object with the extracted data
    let object = ClientObject {
        id: object_id,
        class_name: class_name.to_string(),
        owner_id,
        state: ObjectLifecycleState::Initializing,
        transform,
        properties,
        needs_id_remap: true,
        is_actor,
        replicates,
        components: Vec::new(),
    };
    
    // Register the object
    register_object_internal(object);
    
    // Create a simplified SpawnParams for server request
    let spawn_params = SpawnParams {
        class_name: class_name.to_string(),
        transform,
        owner_id,
        replicates,
        initial_properties,
        is_system: false,
    };
    
    // Only request server creation if replicates is true and we're connected
    if replicates && crate::net::is_connected() {
        request_server_create_object(object_id, class_name, &spawn_params)?;
    }
    
    Ok(object_id)
}

/// Dispatch an unreliable RPC (Remote Procedure Call) for an object
pub fn dispatch_unreliable_rpc(
    object_id: ObjectId,
    function_name: &str,
    params: &serde_json::Value,
) -> Result<(), String> {
    // Check if the object exists
    let object = {
        let objects = CLIENT_OBJECTS.lock().unwrap();
        match objects.get(&object_id) {
            Some(obj) => obj.clone(),
            None => return Err(format!("Object {} not found", object_id))
        }
    };
    
    // Only allow RPCs for replicated objects
    if !object.replicates {
        return Err(format!("Cannot dispatch RPC for non-replicated object {}", object_id));
    }
    
    // Check if we're connected to the server
    if !crate::net::is_connected() {
        return Err("Not connected to server".to_string());
    }
    
    // Prepare the RPC request data
    let request_data = serde_json::json!({
        "object_id": object_id,
        "function": function_name,
        "args": params,
        "reliable": false
    });
    
    // Serialize to JSON
    let request_json = serde_json::to_string(&request_data)
        .map_err(|e| format!("Failed to serialize RPC request: {}", e))?;
    
    // Send the RPC request
    crate::net::send_rpc_request(&request_json)?;
    
    debug!("Dispatched unreliable RPC {} for object {}", function_name, object_id);
    Ok(())
}

/// Set a property on an object with control over replication
pub fn set_object_property_with_replication(
    object_id: ObjectId,
    property_name: &str,
    value: &serde_json::Value,
    replicate: bool,
) -> Result<(), String> {
    // Deserialize the value to a PropertyValue
    let property_value = property::serialization::deserialize_property_value(&value.to_string())?;
    
    // Update locally
    update_object_property(object_id, property_name, property_value.clone())?;
    
    // Replicate to server if needed
    if replicate {
        net::send_property_update(object_id, property_name, &value.to_string())?;
    }
    
    Ok(())
}

/// Get the class name for an object
pub fn get_class_for_object(object_id: ObjectId) -> Option<String> {
    let objects = CLIENT_OBJECTS.lock().unwrap();
    if let Some(obj) = objects.get(&object_id) {
        return Some(obj.class_name.clone());
    }
    None
}

/// Get all registered objects
pub fn get_property(object_id: ObjectId, property_name: &str) -> Result<Option<PropertyValue>, String> {
    let objects = CLIENT_OBJECTS.lock().unwrap();
    
    if let Some(obj) = objects.get(&object_id) {
        return Ok(obj.properties.get(property_name).cloned());
    }
    
    // No object found, check if we have property in the staging cache
    let cache = crate::property::get_cached_property_value(object_id, property_name);
    if cache.is_some() {
        return Ok(cache);
    }
    
    Ok(None)
} 