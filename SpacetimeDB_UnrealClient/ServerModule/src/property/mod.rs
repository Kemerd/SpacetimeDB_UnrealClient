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

use crate::object::{ObjectId, UObject};
use crate::client::ClientInfo;

// Re-export submodules
pub mod conversion;
pub mod validation;
pub mod replication;
pub mod serialization;

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
    let object = match UObject::filter_by_id(&ctx, object_id)
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
    
    // Validate the property value
    // In a full implementation, we would check against property constraints
    
    // Update the object's property
    // In a real implementation, we would access the UObject table
    // and update the property, then track the change for replication
    
    // Log the property change
    ctx.logger().info(format!(
        "Client {} set property '{}' on object {} to a new value",
        client_id, property_name, object_id
    ));
    
    Ok(())
}

/// Check if a client can modify an object
fn can_modify_object(client: &ClientInfo, object: &UObject) -> bool {
    // Admins can modify any object
    if client.is_admin {
        return true;
    }
    
    // Owners can modify their objects
    if object.owner_client_id == client.id {
        return true;
    }
    
    // Objects with no owner can be modified by anyone
    if object.owner_client_id == 0 {
        return true;
    }
    
    // Otherwise, no permission
    false
}

/// Initialize the property system
pub fn init(ctx: &mut StageReducer) {
    ctx.logger().info("Initializing property system...");
    
    // In a real implementation, we would:
    // 1. Load property definitions from configuration
    // 2. Initialize the property registry
    // 3. Set up replication schedules
    
    ctx.logger().info("Property system initialized!");
} 