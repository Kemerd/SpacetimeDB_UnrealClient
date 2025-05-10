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
use serde_json::{Value, json};
use spacetimedb::AlgebraicValue;

// Import submodules
pub mod serialization;

// Re-export serialization functions for convenience
pub use serialization::{serialize_property_value, deserialize_property_value};

// Define our own TableUpdate and TableOp to replace the missing SDK structures
pub struct TableRow {
    pub row: Option<HashMap<String, Value>>,
}

pub struct TableRowOperation {
    pub operation: TableOp,
    pub row: Option<HashMap<String, Value>>,
}

pub enum TableOp {
    Insert,
    Update,
    Delete,
}

pub struct TableUpdate {
    pub table_name: String,
    pub table_row_operations: Vec<TableRowOperation>,
}

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

/// Initialize the property system
pub fn init() {
    debug!("Initializing property system");
    
    // Register handler for property definition table updates
    crate::net::register_table_handler("PropertyDefinition", |update| {
        handle_property_definition_update(update)
    });
    
    debug!("Property system initialized");
}

/// Handle property definition updates from the server
pub fn handle_property_definition_update(update: &TableUpdate) -> Result<(), String> {
    // Process all rows in the update
    for row in &update.table_row_operations {
        if let Some(row_data) = &row.row {
            match row.operation {
                TableOp::Insert => {
                    // Get the property definition from the row data
                    let property_name = match row_data.get("property_name") {
                        Some(Value::String(s)) => s,
                        _ => return Err("Missing property_name in property definition".to_string()),
                    };
                    
                    let property_class = match row_data.get("class_name") {
                        Some(Value::String(s)) => s,
                        _ => return Err("Missing class_name in property definition".to_string()),
                    };
                    
                    let property_type_str = match row_data.get("property_type") {
                        Some(Value::String(s)) => s,
                        _ => return Err("Missing property_type in property definition".to_string()),
                    };
                    
                    let property_type = parse_property_type(&property_type_str)?;
                    
                    let replicated = match row_data.get("replicated") {
                        Some(Value::Bool(b)) => b,
                        _ => false,
                    };
                    
                    let replication_condition_str = match row_data.get("replication_condition") {
                        Some(Value::String(s)) => s.to_string(),
                        _ => "OnChange".to_string(),
                    };
                    
                    let replication_condition = parse_replication_condition(&replication_condition_str)?;
                    
                    let readonly = match row_data.get("readonly") {
                        Some(Value::Bool(b)) => b,
                        _ => false,
                    };
                    
                    let flags = match row_data.get("flags") {
                        Some(Value::Number(n)) => n.as_u64().unwrap_or(0) as u32,
                        _ => 0,
                    };
                    
                    // Create property definition
                    let definition = PropertyDefinition {
                        name: property_name.to_string(),
                        property_type,
                        replicated: *replicated,
                        replication_condition,
                        readonly: *readonly,
                        flags,
                    };
                    
                    // Add to definitions
                    let mut definitions = PROPERTY_DEFINITIONS.lock().unwrap();
                    let class_props = definitions
                        .entry(property_class.to_string())
                        .or_insert_with(HashMap::new);
                    
                    class_props.insert(property_name.to_string(), definition);
                    
                    return Ok(());
                },
                TableOp::Delete => {
                    // Extract property name and class
                    let property_name = match row_data.get("property_name") {
                        Some(Value::String(s)) => s,
                        _ => return Err("Missing property_name in property definition delete".to_string()),
                    };
                    
                    let property_class = match row_data.get("class_name") {
                        Some(Value::String(s)) => s,
                        _ => return Err("Missing class_name in property definition delete".to_string()),
                    };
                    
                    // Remove from definitions
                    let mut definitions = PROPERTY_DEFINITIONS.lock().unwrap();
                    if let Some(class_props) = definitions.get_mut(property_class) {
                        class_props.remove(property_name);
                    }
                    
                    return Ok(());
                },
                _ => return Ok(()),
            }
        }
        
        return Ok(());
    }
    
    Ok(())
}

/// Parse property type from string
fn parse_property_type(type_str: &str) -> Result<PropertyType, String> {
    match type_str {
        "Boolean" => Ok(PropertyType::Boolean),
        "Integer" => Ok(PropertyType::Integer),
        "Float" => Ok(PropertyType::Float),
        "String" => Ok(PropertyType::String),
        "Vector" => Ok(PropertyType::Vector),
        "Rotator" => Ok(PropertyType::Rotator),
        "Transform" => Ok(PropertyType::Transform),
        "Color" => Ok(PropertyType::Color),
        "DateTime" => Ok(PropertyType::DateTime),
        "Object" => Ok(PropertyType::Object),
        "Array" => Ok(PropertyType::Array),
        "Map" => Ok(PropertyType::Map),
        "Set" => Ok(PropertyType::Set),
        "Binary" => Ok(PropertyType::Binary),
        _ => Err(format!("Unknown property type: {}", type_str)),
    }
}

/// Parse replication condition from string
fn parse_replication_condition(condition_str: &str) -> Result<stdb_shared::property::ReplicationCondition, String> {
    match condition_str.as_str() {
        "Initial" => Ok(stdb_shared::property::ReplicationCondition::Initial),
        "OnChange" => Ok(stdb_shared::property::ReplicationCondition::OnChange),
        "Always" => Ok(stdb_shared::property::ReplicationCondition::Always),
        _ => Err(format!("Unknown replication condition: {}", condition_str)),
    }
}

/// Import property definitions from a JSON definition
pub fn import_property_definitions_from_json(json_str: &str) -> Result<usize, String> {
    // Parse the JSON
    let definitions: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse property definitions: {}", e))?;
    
    // Ensure we have an object
    let definitions_obj = match definitions {
        Value::Object(obj) => obj,
        _ => return Err("Property definitions must be a JSON object".to_string()),
    };
    
    let mut count = 0;
    
    // Process each class
    for (class_name, class_props) in definitions_obj {
        let props_obj = match class_props {
            Value::Object(obj) => obj,
            _ => continue, // Skip if not an object
        };
        
        // Process each property in the class
        for (prop_name, prop_def) in props_obj {
            let prop_def_obj = match prop_def {
                Value::Object(obj) => obj,
                _ => continue, // Skip if not an object
            };
            
            // Extract property type
            let prop_type_str = match prop_def_obj.get("type") {
                Some(Value::String(s)) => s,
                _ => continue, // Skip if no type or not a string
            };
            
            // Parse property type
            let prop_type = parse_property_type(prop_type_str)?;
            
            // Get replication flag
            let replicated = match prop_def_obj.get("replicated") {
                Some(Value::Bool(b)) => *b,
                _ => false, // Default to not replicated
            };
            
            // Register the property definition
            register_property_definition(&class_name, &prop_name, prop_type, replicated);
            count += 1;
        }
    }
    
    debug!("Imported {} property definitions from JSON", count);
    Ok(count)
}

/// Export current property definitions as JSON
pub fn export_property_definitions_as_json() -> Result<String, String> {
    let definitions = PROPERTY_DEFINITIONS.lock().unwrap();
    let mut result = json!({});
    
    // Build the JSON structure
    for (class_name, props) in definitions.iter() {
        let mut class_props = json!({});
        
        for (prop_name, def) in props.iter() {
            // Convert property type to string
            let type_str = match def.property_type {
                PropertyType::Bool => "Bool",
                PropertyType::Byte => "Byte",
                PropertyType::Int32 => "Int32",
                PropertyType::Int64 => "Int64",
                PropertyType::UInt32 => "UInt32",
                PropertyType::UInt64 => "UInt64",
                PropertyType::Float => "Float",
                PropertyType::Double => "Double",
                PropertyType::String => "String",
                PropertyType::Vector => "Vector",
                PropertyType::Rotator => "Rotator",
                PropertyType::Quat => "Quat",
                PropertyType::Transform => "Transform",
                PropertyType::Color => "Color",
                PropertyType::ObjectReference => "ObjectReference",
                PropertyType::ClassReference => "ClassReference",
                PropertyType::Array => "Array",
                PropertyType::Map => "Map",
                PropertyType::Set => "Set",
                PropertyType::Name => "Name",
                PropertyType::Text => "Text",
                PropertyType::Custom => "Custom",
                PropertyType::None => "None",
            };
            
            // Create property definition JSON
            let prop_def = json!({
                "type": type_str,
                "replicated": def.replicated,
                "readonly": def.readonly,
                "replication_condition": format!("{:?}", def.replication_condition),
                "flags": def.flags,
            });
            
            // Add to class properties
            if let Some(obj) = class_props.as_object_mut() {
                obj.insert(prop_name.clone(), prop_def);
            }
        }
        
        // Add class properties to result
        if let Some(obj) = result.as_object_mut() {
            obj.insert(class_name.clone(), class_props);
        }
    }
    
    // Serialize to string
    serde_json::to_string_pretty(&result)
        .map_err(|e| format!("Failed to serialize property definitions: {}", e))
}

/// Get all registered class names
pub fn get_registered_class_names() -> Vec<String> {
    let definitions = PROPERTY_DEFINITIONS.lock().unwrap();
    definitions.keys().cloned().collect()
}

/// Get all property names for a given class
pub fn get_property_names_for_class(class_name: &str) -> Vec<String> {
    let definitions = PROPERTY_DEFINITIONS.lock().unwrap();
    
    match definitions.get(class_name) {
        Some(props) => props.keys().cloned().collect(),
        None => Vec::new(),
    }
}

/// Check if property definitions are loaded for a class
pub fn has_property_definitions_for_class(class_name: &str) -> bool {
    let definitions = PROPERTY_DEFINITIONS.lock().unwrap();
    
    match definitions.get(class_name) {
        Some(props) => !props.is_empty(),
        None => false,
    }
}

/// Get the number of registered property definitions
pub fn get_property_definition_count() -> usize {
    let definitions = PROPERTY_DEFINITIONS.lock().unwrap();
    let mut count = 0;
    
    for props in definitions.values() {
        count += props.len();
    }
    
    count
}

/// Add a property to a class definition
pub fn add_property(
    class_name: &str,
    property_name: &str,
    type_name: &str,
    replicated: bool,
    replication_condition: stdb_shared::property::ReplicationCondition,
    readonly: bool,
    flags: u32,
) -> bool {
    // Convert the type_name string to a PropertyType
    let property_type = match type_name {
        "Bool" => PropertyType::Bool,
        "Byte" => PropertyType::Byte,
        "Int32" => PropertyType::Int32,
        "Int64" => PropertyType::Int64,
        "UInt32" => PropertyType::UInt32,
        "UInt64" => PropertyType::UInt64,
        "Float" => PropertyType::Float,
        "Double" => PropertyType::Double,
        "String" => PropertyType::String,
        "Vector" => PropertyType::Vector,
        "Rotator" => PropertyType::Rotator,
        "Quat" => PropertyType::Quat,
        "Transform" => PropertyType::Transform,
        "Color" => PropertyType::Color,
        "ObjectReference" => PropertyType::ObjectReference,
        "ClassReference" => PropertyType::ClassReference,
        "Array" => PropertyType::Array,
        "Map" => PropertyType::Map,
        "Set" => PropertyType::Set,
        "Name" => PropertyType::Name,
        "Text" => PropertyType::Text,
        "Custom" => PropertyType::Custom,
        _ => {
            warn!("Unknown property type: {}", type_name);
            return false;
        }
    };
    
    // Create the property definition
    let definition = PropertyDefinition {
        name: property_name.to_string(),
        property_type,
        replicated,
        replication_condition,
        readonly,
        flags,
    };
    
    // Add the property definition to the class
    let mut definitions = PROPERTY_DEFINITIONS.lock().unwrap();
    
    // Get or create the class property map
    let class_props = definitions
        .entry(class_name.to_string())
        .or_insert_with(HashMap::new);
    
    // Check if property already exists
    if class_props.contains_key(property_name) {
        return false;
    }
    
    // Add to map
    class_props.insert(property_name.to_string(), definition);
    
    debug!("Added property {} to class {}", property_name, class_name);
    true
}

/// Unregister a property definition
pub fn unregister_property_definition(class_name: &str, property_name: &str) -> bool {
    let mut definitions = PROPERTY_DEFINITIONS.lock().unwrap();
    
    if let Some(class_props) = definitions.get_mut(class_name) {
        let removed = class_props.remove(property_name).is_some();
        if removed {
            debug!("Unregistered property definition: {}.{}", class_name, property_name);
        }
        return removed;
    }
    
    false
}

/// Implement the helper functions that were referenced but missing
pub fn get_property_names_for_class_as_json(class_name: &str) -> Result<String, String> {
    let property_names = get_property_names_for_class(class_name);
    serde_json::to_string(&property_names)
        .map_err(|e| format!("Failed to serialize property names: {}", e))
}

pub fn get_registered_class_names_as_json() -> Result<String, String> {
    let class_names = get_registered_class_names();
    serde_json::to_string(&class_names)
        .map_err(|e| format!("Failed to serialize class names: {}", e))
}

pub fn get_property_definition_as_json(class_name: &str, property_name: &str) -> Result<String, String> {
    if let Some(definition) = get_property_definition(class_name, property_name) {
        serde_json::to_string(&definition)
            .map_err(|e| format!("Failed to serialize property definition: {}", e))
    } else {
        Ok("{}".to_string())
    }
} 