//! # Client-side Property System
//!
//! Handles client-side property management, serialization, and synchronization.

use stdb_shared::property::{PropertyType, PropertyValue, PropertyDefinition, ReplicationCondition};
use stdb_shared::object::ObjectId;
use stdb_shared::types::*;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

pub mod serialization;  // Re-export from shared
pub mod conversion;     // Client-specific conversion utilities
pub mod cache;          // Client-side property cache

// Re-export from shared for convenience
pub use stdb_shared::property::{PropertyType, PropertyValue, ReplicationCondition};

/// Client-side cache of property definitions
static PROPERTY_DEFINITIONS: Lazy<Mutex<HashMap<String, HashMap<String, PropertyDefinition>>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Client-side cache of property values
static PROPERTY_CACHE: Lazy<Mutex<HashMap<ObjectId, HashMap<String, PropertyValue>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Initialize the property system
pub fn init() {
    println!("Initializing client-side property system");
    
    // Register default property definitions
    let mut defs = PROPERTY_DEFINITIONS.lock().unwrap();
    register_default_property_definitions(&mut defs);
    
    println!("Client-side property system initialized!");
}

/// Register default property definitions for common classes
fn register_default_property_definitions(
    defs: &mut HashMap<String, HashMap<String, PropertyDefinition>>
) {
    // Default properties for Actor class
    let mut actor_props = HashMap::new();
    
    // Transform properties
    actor_props.insert(
        "Location".to_string(),
        PropertyDefinition {
            name: "Location".to_string(),
            property_type: PropertyType::Vector,
            replicated: true,
            replication_condition: ReplicationCondition::OnChange,
            readonly: false,
            flags: 0,
        }
    );
    
    actor_props.insert(
        "Rotation".to_string(),
        PropertyDefinition {
            name: "Rotation".to_string(),
            property_type: PropertyType::Rotator,
            replicated: true,
            replication_condition: ReplicationCondition::OnChange,
            readonly: false,
            flags: 0,
        }
    );
    
    actor_props.insert(
        "Scale".to_string(),
        PropertyDefinition {
            name: "Scale".to_string(),
            property_type: PropertyType::Vector,
            replicated: true,
            replication_condition: ReplicationCondition::OnChange,
            readonly: false,
            flags: 0,
        }
    );
    
    // Add Actor properties to definitions
    defs.insert("Actor".to_string(), actor_props);
    
    // Add additional class definitions as needed
}

/// Register a property definition for a class
pub fn register_property_definition(
    class_name: &str,
    property_name: &str,
    definition: PropertyDefinition,
) {
    let mut defs = PROPERTY_DEFINITIONS.lock().unwrap();
    
    // Get or create class property map
    let class_props = defs
        .entry(class_name.to_string())
        .or_insert_with(HashMap::new);
    
    // Add/update property definition
    class_props.insert(property_name.to_string(), definition);
}

/// Get a property definition
pub fn get_property_definition(
    class_name: &str,
    property_name: &str,
) -> Option<PropertyDefinition> {
    let defs = PROPERTY_DEFINITIONS.lock().unwrap();
    
    // Look up class
    defs.get(class_name)
        .and_then(|class_props| {
            // Look up property
            class_props.get(property_name).cloned()
        })
}

/// Cache a property value for an object
pub fn cache_property_value(
    object_id: ObjectId,
    property_name: &str,
    value: PropertyValue,
) {
    let mut cache = PROPERTY_CACHE.lock().unwrap();
    
    // Get or create object property map
    let obj_props = cache
        .entry(object_id)
        .or_insert_with(HashMap::new);
    
    // Add/update property value
    obj_props.insert(property_name.to_string(), value);
}

/// Get a cached property value
pub fn get_cached_property_value(
    object_id: ObjectId,
    property_name: &str,
) -> Option<PropertyValue> {
    let cache = PROPERTY_CACHE.lock().unwrap();
    
    // Look up object
    cache.get(&object_id)
        .and_then(|obj_props| {
            // Look up property
            obj_props.get(property_name).cloned()
        })
}

/// Clear cached properties for an object
pub fn clear_object_cache(object_id: ObjectId) {
    let mut cache = PROPERTY_CACHE.lock().unwrap();
    cache.remove(&object_id);
}

/// Update multiple properties at once from a JSON map
pub fn update_properties_from_json(
    object_id: ObjectId,
    properties_json: &str,
) -> Result<(), String> {
    // Parse JSON into property map
    let property_map: HashMap<String, String> = serde_json::from_str(properties_json)
        .map_err(|e| format!("Failed to parse properties JSON: {}", e))?;
    
    // Update each property
    for (name, value_json) in property_map {
        let property_value = serialization::deserialize_property_value(&value_json)?;
        cache_property_value(object_id, &name, property_value);
    }
    
    Ok(())
} 