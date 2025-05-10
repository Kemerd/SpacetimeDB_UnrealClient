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
use spacetimedb_sdk::messages::TableType;
use spacetimedb_sdk::{identity, Address, Identity, Timestamp};
use std::collections::HashMap;
use std::sync::RwLock;

use crate::object::{ObjectId, ObjectInstance, ObjectProperty};
use crate::connection::ClientInfo;
use stdb_shared::property::{PropertyType, PropertyValue, PropertyDefinition, ReplicationCondition};
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

/// Table to store property definitions that are shared with clients
#[derive(TableType)]
pub struct PropertyDefinitionTable {
    #[primarykey]
    /// Combined class_name + property_name
    pub id: String,
    /// The class name this property belongs to
    pub class_name: String,
    /// The property name
    pub property_name: String,
    /// String representation of property type
    pub property_type: String,
    /// Whether property is replicated
    pub replicated: bool,
    /// How the property replicates (as string)
    pub replication_condition: String,
    /// Whether property is read-only for clients
    pub readonly: bool,
    /// Additional flags
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
    ctx.logger().info("Initializing property system");
    
    // Load property definitions
    load_property_definitions(ctx);
    
    // Sync the property registry to the PropertyDefinitionTable for client access
    sync_property_definitions_to_table(ctx);
    
    // Set up periodic replication
    schedule_replication(ctx);
    
    ctx.logger().info("Property system initialized");
}

/// Load property definitions into the registry
fn load_property_definitions(ctx: &StageReducer) {
    let mut registry = match PROPERTY_REGISTRY.write() {
        Ok(registry) => registry,
        Err(e) => {
            ctx.logger().error(format!("Failed to acquire registry lock: {}", e));
            return;
        }
    };
    
    // Clear existing definitions
    registry.clear();
    
    // Add standard engine properties
    add_standard_properties(&mut registry);
    
    // Add game-specific custom properties
    add_custom_properties(ctx, &mut registry);
    
    // Include generated property definitions
    if let Err(e) = crate::generated::register_property_definitions(&mut registry) {
        ctx.logger().warn(format!("Error loading generated property definitions: {}", e));
    }
    
    ctx.logger().info(format!("Loaded {} property definitions", registry.len()));
}

/// Synchronize the property registry to the PropertyDefinitionTable
fn sync_property_definitions_to_table(ctx: &StageReducer) {
    // Get a read lock on the registry
    let registry = match PROPERTY_REGISTRY.read() {
        Ok(registry) => registry,
        Err(e) => {
            ctx.logger().error(format!("Failed to acquire registry lock: {}", e));
            return;
        }
    };
    
    // Count for logging
    let mut count = 0;
    
    // Group properties by class
    let mut class_properties: HashMap<String, Vec<(&String, &PropertyDefinition)>> = HashMap::new();
    
    for (name, def) in registry.iter() {
        // Extract class name from property path (Class.Property)
        let parts: Vec<&str> = name.split('.').collect();
        if parts.len() == 2 {
            let class_name = parts[0].to_string();
            let property_entries = class_properties.entry(class_name).or_insert_with(Vec::new);
            property_entries.push((name, def));
        } else {
            // For properties without class prefix, use "Common"
            let property_entries = class_properties.entry("Common".to_string()).or_insert_with(Vec::new);
            property_entries.push((name, def));
        }
    }
    
    // Process each class
    for (class_name, properties) in class_properties.iter() {
        for (full_name, def) in properties {
            // Extract property name from full name
            let property_name = match full_name.split('.').last() {
                Some(name) => name,
                None => full_name.as_str(), // Use full name as fallback
            };
            
            // Convert property type to string
            let type_str = format!("{:?}", def.property_type);
            
            // Convert replication condition to string
            let condition_str = format!("{:?}", def.replication_condition);
            
            // Create unique ID (class_name:property_name)
            let id = format!("{}:{}", class_name, property_name);
            
            // Create table entry
            let table_entry = PropertyDefinitionTable {
                id,
                class_name: class_name.clone(),
                property_name: property_name.to_string(),
                property_type: type_str,
                replicated: def.replicated,
                replication_condition: condition_str,
                readonly: def.readonly,
                flags: def.flags,
            };
            
            // Insert or update in table
            match PropertyDefinitionTable::filter_by_id(ctx, &table_entry.id).into_iter().next() {
                Some(_existing) => {
                    // Update existing entry
                    if let Err(e) = PropertyDefinitionTable::update(ctx, table_entry) {
                        ctx.logger().warn(format!("Failed to update property definition {}: {}", full_name, e));
                        continue;
                    }
                },
                None => {
                    // Insert new entry
                    if let Err(e) = PropertyDefinitionTable::insert(ctx, table_entry) {
                        ctx.logger().warn(format!("Failed to insert property definition {}: {}", full_name, e));
                        continue;
                    }
                }
            }
            
            count += 1;
        }
    }
    
    ctx.logger().info(format!("Synchronized {} property definitions to client-accessible table", count));
}

/// Register a property definition
pub fn register_property_definition(
    registry: &mut HashMap<String, PropertyDefinition>,
    full_name: &str,
    property_type: PropertyType,
    replicated: bool,
    readonly: bool,
    replication_condition: ReplicationCondition,
    flags: u32,
) {
    // Create definition
    let definition = PropertyDefinition {
        name: full_name.to_string(),
        property_type,
        replicated,
        replication_condition,
        readonly,
        flags,
    };
    
    // Add to registry
    registry.insert(full_name.to_string(), definition);
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