//! # Property Module
//!
//! Handles client-side property definitions and values for UObjects.
//! This module provides the client-side interface for working with 
//! property values and maintains a staging cache for object properties
//! that haven't yet been associated with specific objects.

use std::sync::Mutex;
use std::collections::HashMap;
use once_cell::sync::Lazy;
use stdb_shared::object::ObjectId;
use stdb_shared::property::{PropertyType, PropertyValue, PropertyDefinition};
use log::{debug, trace, warn};

// Import submodules
pub mod serialization;

// Re-export serialization functions for convenience
pub use serialization::{serialize_property_value, deserialize_property_value};

// Property staging cache
// This cache serves as a temporary storage for property values that:
// 1. Arrive from the server before their owning object is created
// 2. Are in transit during object creation
// 3. Need to be preserved during ID remapping
// Once associated with an object, properties should be moved to the object's own properties map.
static PROPERTY_CACHE: Lazy<Mutex<HashMap<ObjectId, HashMap<String, PropertyValue>>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

// Property definitions
// Maps ClassName -> PropertyName -> PropertyDefinition
static PROPERTY_DEFINITIONS: Lazy<Mutex<HashMap<String, HashMap<String, PropertyDefinition>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Register a property definition
pub fn register_property_definition(
    class_name: &str,
    property_name: &str,
    property_type: PropertyType,
    replicated: bool,
) -> bool {
    let mut definitions = PROPERTY_DEFINITIONS.lock().unwrap();
    
    // Get or create the class property map
    let class_props = definitions
        .entry(class_name.to_string())
        .or_insert_with(HashMap::new);
    
    // Check if property already exists
    if class_props.contains_key(property_name) {
        return false;
    }
    
    // Create property definition
    let definition = PropertyDefinition {
        name: property_name.to_string(),
        property_type,
        replicated,
        replication_condition: stdb_shared::property::ReplicationCondition::OnChange,
        readonly: false,
        flags: 0,
    };
    
    // Add to map
    class_props.insert(property_name.to_string(), definition);
    
    true
}

/// Get a property definition
pub fn get_property_definition(
    class_name: &str,
    property_name: &str,
) -> Option<PropertyDefinition> {
    let definitions = PROPERTY_DEFINITIONS.lock().unwrap();
    
    // Get class properties
    definitions.get(class_name)
        .and_then(|props| props.get(property_name))
        .cloned()
}

/// Temporarily cache a property value before it's associated with an object
/// 
/// This function is primarily used in three scenarios:
/// 1. When properties arrive from the server before the object itself is created
/// 2. During object creation when properties need to be set before the object is fully initialized
/// 3. During ID remapping to preserve property values
///
/// IMPORTANT: For normal property updates on existing objects, use object::update_object_property() instead,
/// which will handle updating both the client object and this cache if needed.
pub fn cache_property_value(
    object_id: ObjectId,
    property_name: &str,
    value: PropertyValue,
) {
    let mut cache = PROPERTY_CACHE.lock().unwrap();
    
    // Get or create the object property map
    let obj_props = cache
        .entry(object_id)
        .or_insert_with(HashMap::new);
    
    // Update property value
    obj_props.insert(property_name.to_string(), value);
}

/// Get a cached property value from the staging cache
/// 
/// This should mainly be used for properties that haven't been associated with 
/// an object yet. For properties of existing objects, prefer to access the object's
/// properties directly through the object module.
pub fn get_cached_property_value(
    object_id: ObjectId,
    property_name: &str,
) -> Option<PropertyValue> {
    let cache = PROPERTY_CACHE.lock().unwrap();
    
    // Get object properties
    cache.get(&object_id)
        .and_then(|props| props.get(property_name))
        .cloned()
}

/// Move all cached properties for an object to the object's own property map
/// 
/// This should be called when an object is fully created to transfer any properties
/// that were cached before the object existed.
pub fn transfer_cached_properties_to_object(object_id: ObjectId) -> HashMap<String, PropertyValue> {
    let mut cache = PROPERTY_CACHE.lock().unwrap();
    
    // Remove and return the properties for this object
    cache.remove(&object_id).unwrap_or_default()
}

/// Clear property cache for an object
pub fn clear_property_cache(object_id: ObjectId) {
    let mut cache = PROPERTY_CACHE.lock().unwrap();
    cache.remove(&object_id);
}

/// Clear property cache for an object (alias for clear_property_cache)
pub fn clear_object_properties(object_id: ObjectId) {
    clear_property_cache(object_id);
}

/// Remap property cache from a temporary ID to a server-assigned ID
pub fn remap_property_cache(temp_id: ObjectId, server_id: ObjectId) {
    let mut cache = PROPERTY_CACHE.lock().unwrap();
    
    // Get properties for the temporary object ID
    if let Some(properties) = cache.remove(&temp_id) {
        // Add properties with the new server ID
        cache.insert(server_id, properties);
        debug!("Remapped property cache from temporary ID {} to server ID {}", temp_id, server_id);
    } else {
        trace!("No properties found for temporary ID {} during remapping", temp_id);
    }
}

/// Cache a property value from a JSON string
pub fn cache_property_value_from_json(
    object_id: ObjectId,
    property_name: &str,
    json_value: &str,
) -> Result<(), String> {
    // Parse JSON to PropertyValue
    let value = serialization::deserialize_property_value(json_value)?;
    
    // Cache the value
    cache_property_value(object_id, property_name, value);
    
    Ok(())
}

/// Get a cached property value as a JSON string
pub fn get_cached_property_value_as_json(
    object_id: ObjectId,
    property_name: &str,
) -> Option<String> {
    // Get the cached value
    get_cached_property_value(object_id, property_name)
        .and_then(|value| {
            serialization::serialize_property_value(&value).ok()
        })
} 