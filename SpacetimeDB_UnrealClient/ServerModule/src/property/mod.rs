//! # Property System
//!
//! The property system manages UObject property types, values, and replication.
//! It provides the foundation for Unreal Engine-like property management within SpacetimeDB.
//!
//! Key features:
//! - Various property types matching Unreal Engine's property system
//! - Property value storage and manipulation
//! - Validation and constraints on property values
//! - Property replication between server and clients
//! - Conversion utilities for different data formats

use spacetimedb_sdk::reducer::StageReducer;
use spacetimedb_sdk::table::TableType;
use spacetimedb_sdk::{identity, Address, Identity, Timestamp};
use std::collections::HashMap;
use std::sync::RwLock;

use crate::object::{ObjectId, ObjectInstance, ObjectProperty};
use crate::connection::ClientInfo;
use crate::SharedModule::property::{PropertyType, PropertyValue, PropertyDefinition, ReplicationCondition};
use crate::relevancy::determine_relevance;

// Re-export submodules
pub mod conversion;
pub mod validation;
pub mod replication;
pub mod serialization;

// A global registry of property definitions - lazily initialized
lazy_static::lazy_static! {
    pub static ref PROPERTY_REGISTRY: RwLock<HashMap<String, PropertyDefinition>> = RwLock::new(HashMap::new());
    pub static ref REPLICATION_QUEUE: RwLock<Vec<QueuedPropertyReplication>> = RwLock::new(Vec::new());
}

/// Structure to track a property change for replication
pub struct QueuedPropertyReplication {
    /// The object that had a property change
    pub object_id: ObjectId,
    /// The property that changed
    pub property_name: String,
    /// When the change occurred
    pub timestamp: u64,
    /// Whether this is high priority
    pub is_high_priority: bool,
}

/// Represents the different types of properties that can be stored in a UObject
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyType {
    // Primitive types
    Bool,
    Byte,
    Int32,
    Int64,
    UInt32,
    UInt64,
    Float,
    Double,
    String,
    
    // Structured types
    Vector,       // FVector (x, y, z)
    Rotator,      // FRotator (pitch, yaw, roll)
    Quat,         // FQuat (x, y, z, w)
    Transform,    // FTransform (position, rotation, scale)
    Color,        // FColor (r, g, b, a)
    
    // Reference types
    ObjectReference,  // Reference to another UObject
    ClassReference,   // Reference to a class
    
    // Container types
    Array,            // TArray<T> - stored as JSON
    Map,              // TMap<K,V> - stored as JSON
    Set,              // TSet<T> - stored as JSON
    
    // Special types
    Name,             // FName
    Text,             // FText
    Custom,           // Custom struct - stored as JSON
}

/// Represents the value of a property
#[derive(Debug, Clone)]
pub enum PropertyValue {
    // Primitive values
    Bool(bool),
    Byte(u8),
    Int32(i32),
    Int64(i64),
    UInt32(u32),
    UInt64(u64),
    Float(f32),
    Double(f64),
    String(String),
    
    // Structured values
    Vector { x: f32, y: f32, z: f32 },
    Rotator { pitch: f32, yaw: f32, roll: f32 },
    Quat { x: f32, y: f32, z: f32, w: f32 },
    Transform {
        pos_x: f32, pos_y: f32, pos_z: f32,
        rot_x: f32, rot_y: f32, rot_z: f32, rot_w: f32,
        scale_x: f32, scale_y: f32, scale_z: f32,
    },
    Color { r: u8, g: u8, b: u8, a: u8 },
    
    // Reference values
    ObjectReference(ObjectId),
    ClassReference(String),  // Class name
    
    // Container values (stored as JSON strings for simplicity)
    ArrayJson(String),
    MapJson(String),
    SetJson(String),
    
    // Special values
    Name(String),
    Text(String),
    CustomJson(String),
    
    // Null value
    None,
}

/// Table to store property definitions
#[derive(TableType)]
pub struct PropertyDefinition {
    /// The ID of the property definition
    pub id: u64,
    /// The name of the property
    pub name: String,
    /// The class name this property belongs to
    pub class_name: String,
    /// The property type
    pub property_type: String,  // Serialized PropertyType
    /// Whether the property is replicated
    pub is_replicated: bool,
    /// The replication condition (if replicated)
    pub replication_condition: String,  // Serialized ReplicationCondition
    /// Property flags
    pub flags: u32,
}

/// Reducer for setting a property value on an object
#[reducer]
pub fn set_object_property(
    ctx: StageReducer,
    client_id: u64,
    object_id: ObjectId,
    property_name: String,
    value: String,  // JSON-serialized PropertyValue
) -> Result<(), String> {
    // Get the client info
    let client_info = match ClientInfo::filter_by_id(&ctx, client_id)
        .into_iter()
        .next() 
    {
        Some(client) => client,
        None => return Err("Client not found".to_string()),
    };
    
    // Get the object
    let object = match ObjectInstance::filter_by_id(&ctx, object_id)
        .into_iter()
        .next()
    {
        Some(obj) => obj,
        None => return Err(format!("Object {} not found", object_id)),
    };
    
    // Check if the client has permission to modify this object
    if !can_modify_object(&client_info, &object) {
        return Err("You don't have permission to modify this object".to_string());
    }
    
    // Parse the value from JSON
    let property_value = match serialization::deserialize_property_value(&value) {
        Ok(val) => val,
        Err(err) => return Err(format!("Failed to parse property value: {}", err)),
    };
    
    // Check property constraints and validate the value
    if let Err(validation_error) = validate_property_value(&ctx, &object, &property_name, &property_value) {
        return Err(format!("Property validation failed: {}", validation_error));
    }
    
    // Get the current property (if it exists)
    let existing_property = ObjectProperty::filter_by_object_and_name(&ctx, object_id, &property_name)
        .into_iter()
        .next();
    
    // Update the object property in the database
    let current_time = ctx.timestamp().as_millis();
    
    if let Some(mut existing) = existing_property {
        // Update existing property
        existing.value = property_value.clone();
        existing.last_updated = current_time;
        
        // Save the updated property
        ObjectProperty::update(&ctx, existing)?;
    } else {
        // Create new property entry
        let property = ObjectProperty {
            object_id,
            property_name: property_name.clone(),
            value: property_value.clone(),
            last_updated: current_time,
            replicated: true, // Default to replicated, can be configured
        };
        
        // Insert the new property
        ObjectProperty::insert(&ctx, property)?;
    }
    
    // Add to replication queue for sending to relevant clients
    queue_property_for_replication(&ctx, object_id, &property_name, current_time);
    
    // Log the property change
    ctx.logger().info(format!(
        "Client {} set property '{}' on object {} to a new value",
        client_id, property_name, object_id
    ));
    
    Ok(())
}

/// Check if a client can modify an object
fn can_modify_object(client: &ClientInfo, object: &ObjectInstance) -> bool {
    // Admins can modify any object
    if client.is_admin {
        return true;
    }
    
    // Owners can modify their objects
    if let Some(owner_identity) = &object.owner_identity {
        if owner_identity == &client.identity {
            return true;
        }
    }
    
    // Objects with no owner can be modified by anyone
    if object.owner_identity.is_none() {
        return true;
    }
    
    // Otherwise, no permission
    false
}

/// Add a property change to the replication queue
fn queue_property_for_replication(ctx: &StageReducer, object_id: ObjectId, property_name: &str, timestamp: u64) {
    // Get property definition to check replication settings
    let replicate = match PROPERTY_REGISTRY.read() {
        Ok(registry) => {
            match registry.get(property_name) {
                Some(def) => def.replicated,
                None => true // Default to replicate if definition not found
            }
        },
        Err(_) => true // Default to replicate if registry lock fails
    };

    if !replicate {
        return;
    }

    // Determine if this is a high priority property (like transform)
    let is_high_priority = property_name.contains("Location") || 
                         property_name.contains("Rotation") ||
                         property_name.contains("Transform");
    
    // Add to queue
    if let Ok(mut queue) = REPLICATION_QUEUE.write() {
        queue.push(QueuedPropertyReplication {
            object_id,
            property_name: property_name.to_string(),
            timestamp,
            is_high_priority,
        });
    } else {
        ctx.logger().warn(format!(
            "Failed to queue property '{}' on object {} for replication",
            property_name, object_id
        ));
    }
}

/// Validate a property value against constraints
fn validate_property_value(
    ctx: &StageReducer, 
    object: &ObjectInstance, 
    property_name: &str, 
    value: &PropertyValue
) -> Result<(), String> {
    // Get property definition if available
    let definition = match PROPERTY_REGISTRY.read() {
        Ok(registry) => registry.get(property_name).cloned(),
        Err(_) => None,
    };
    
    // Basic type validation
    if let Some(def) = &definition {
        if def.property_type != value.get_type() {
            return Err(format!(
                "Type mismatch: expected {:?}, got {:?}",
                def.property_type, value.get_type()
            ));
        }
    }
    
    // Validate specific property types
    match value {
        PropertyValue::ObjectReference(ref_id) => {
            // Check that referenced object exists
            if ObjectInstance::filter_by_id(ctx, *ref_id).is_empty() {
                return Err(format!("Referenced object {} does not exist", ref_id));
            }
        },
        PropertyValue::Vector(v) => {
            // Example validation: check for NaN values
            if v.x.is_nan() || v.y.is_nan() || v.z.is_nan() {
                return Err("Vector contains NaN values".to_string());
            }
        },
        // Add other validations as needed for specific types
        _ => {} // Other types might not need special validation
    }
    
    // For custom validation logic, call validation module functions
    if let Err(e) = validation::validate_property(object, property_name, value) {
        return Err(e);
    }
    
    Ok(())
}

/// Initialize the property system
pub fn init(ctx: &mut StageReducer) {
    ctx.logger().info("Initializing property system...");
    
    // Load property definitions from configuration
    load_property_definitions(ctx);
    
    // Schedule periodic replication task
    schedule_replication(ctx);
    
    ctx.logger().info("Property system initialized!");
}

/// Load property definitions from configuration or predefined sets
fn load_property_definitions(ctx: &StageReducer) {
    ctx.logger().info("Loading property definitions...");
    
    // Create a new registry (or clear existing)
    if let Ok(mut registry) = PROPERTY_REGISTRY.write() {
        registry.clear();
        
        // Add standard property definitions
        add_standard_properties(&mut registry);
        
        // Load custom properties from configuration
        // This could be loaded from a file, database, or other configuration source
        add_custom_properties(ctx, &mut registry);
        
        ctx.logger().info(format!("Loaded {} property definitions", registry.len()));
    } else {
        ctx.logger().error("Failed to acquire write lock for property registry");
    }
}

/// Add standard Unreal Engine property definitions
fn add_standard_properties(registry: &mut HashMap<String, PropertyDefinition>) {
    // Add transform properties
    let location_def = PropertyDefinition {
        name: "Location".to_string(),
        property_type: PropertyType::Vector,
        replicated: true,
        replication_condition: ReplicationCondition::OnChange,
        readonly: false,
        flags: 0,
    };
    registry.insert("Location".to_string(), location_def);
    
    let rotation_def = PropertyDefinition {
        name: "Rotation".to_string(),
        property_type: PropertyType::Rotator,
        replicated: true,
        replication_condition: ReplicationCondition::OnChange,
        readonly: false,
        flags: 0,
    };
    registry.insert("Rotation".to_string(), rotation_def);
    
    let scale_def = PropertyDefinition {
        name: "Scale".to_string(),
        property_type: PropertyType::Vector,
        replicated: true,
        replication_condition: ReplicationCondition::OnChange,
        readonly: false,
        flags: 0,
    };
    registry.insert("Scale".to_string(), scale_def);
    
    // Add other common properties (health, name, etc.)
    let health_def = PropertyDefinition {
        name: "Health".to_string(),
        property_type: PropertyType::Float,
        replicated: true,
        replication_condition: ReplicationCondition::OnChange,
        readonly: false,
        flags: 0,
    };
    registry.insert("Health".to_string(), health_def);
    
    let display_name_def = PropertyDefinition {
        name: "DisplayName".to_string(),
        property_type: PropertyType::String,
        replicated: true,
        replication_condition: ReplicationCondition::Initial,
        readonly: false,
        flags: 0,
    };
    registry.insert("DisplayName".to_string(), display_name_def);
}

/// Add custom properties from configuration
fn add_custom_properties(ctx: &StageReducer, registry: &mut HashMap<String, PropertyDefinition>) {
    // In a real implementation, this would load from a configuration file or database
    // For now, we'll just add a few example custom properties
    
    let custom_property_def = PropertyDefinition {
        name: "CustomData".to_string(),
        property_type: PropertyType::CustomJson,
        replicated: true,
        replication_condition: ReplicationCondition::OnChange,
        readonly: false,
        flags: 0,
    };
    registry.insert("CustomData".to_string(), custom_property_def);
}

/// Set up scheduled replication of properties
fn schedule_replication(ctx: &StageReducer) {
    // In a real implementation, this would set up periodic tasks to process the replication queue
    // For SpacetimeDB, we'd need to create a reducer that's called on a timer or in response to events
    
    ctx.logger().info("Scheduled property replication system initialized");
}

/// Process the property replication queue (called periodically)
#[reducer]
pub fn process_property_replication_queue(ctx: StageReducer) {
    let current_time = ctx.timestamp().as_millis();
    
    // Get queued property changes
    let properties_to_replicate = {
        if let Ok(mut queue) = REPLICATION_QUEUE.write() {
            // Take ownership of the queue and replace with empty
            std::mem::take(&mut *queue)
        } else {
            ctx.logger().error("Failed to acquire write lock for replication queue");
            return;
        }
    };
    
    if properties_to_replicate.is_empty() {
        return;
    }
    
    ctx.logger().info(format!("Processing {} property changes for replication", properties_to_replicate.len()));
    
    // Group by object for efficiency
    let mut by_object: HashMap<ObjectId, Vec<(String, u64, bool)>> = HashMap::new();
    
    for prop in properties_to_replicate {
        by_object
            .entry(prop.object_id)
            .or_default()
            .push((prop.property_name, prop.timestamp, prop.is_high_priority));
    }
    
    // For each object, replicate to relevant clients
    for (object_id, properties) in by_object {
        // Get the object
        if let Some(object) = ObjectInstance::filter_by_id(&ctx, object_id).into_iter().next() {
            // Get relevant clients for this object
            let relevant_clients = determine_relevance(&ctx, &object);
            
            // For each property, send to relevant clients
            for (property_name, timestamp, is_high_priority) in properties {
                // Get the property value
                if let Some(property) = ObjectProperty::filter_by_object_and_name(&ctx, object_id, &property_name)
                    .into_iter()
                    .next() 
                {
                    // Send to each relevant client
                    for client_id in &relevant_clients {
                        replication::send_property_update(
                            &ctx,
                            *client_id,
                            object_id,
                            &property_name,
                            &property.value,
                            is_high_priority,
                        );
                    }
                }
            }
        }
    }
} 