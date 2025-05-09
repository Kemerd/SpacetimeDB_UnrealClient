//! # Shared Property System
//!
//! Common property types and values used across client and server.

use serde::{Serialize, Deserialize};
use crate::types::*;
use crate::object::ObjectId;
use std::collections::HashMap;

/// Represents the different types of properties that can be stored in a UObject
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    Vector(Vector3),
    Rotator(Rotator),
    Quat(Quat),
    Transform(Transform),
    Color(Color),
    
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

impl PropertyValue {
    /// Get the type of this property value
    pub fn get_type(&self) -> PropertyType {
        match self {
            Self::Bool(_) => PropertyType::Bool,
            Self::Byte(_) => PropertyType::Byte,
            Self::Int32(_) => PropertyType::Int32,
            Self::Int64(_) => PropertyType::Int64,
            Self::UInt32(_) => PropertyType::UInt32,
            Self::UInt64(_) => PropertyType::UInt64,
            Self::Float(_) => PropertyType::Float,
            Self::Double(_) => PropertyType::Double,
            Self::String(_) => PropertyType::String,
            Self::Vector(_) => PropertyType::Vector,
            Self::Rotator(_) => PropertyType::Rotator,
            Self::Quat(_) => PropertyType::Quat,
            Self::Transform(_) => PropertyType::Transform,
            Self::Color(_) => PropertyType::Color,
            Self::ObjectReference(_) => PropertyType::ObjectReference,
            Self::ClassReference(_) => PropertyType::ClassReference,
            Self::ArrayJson(_) => PropertyType::Array,
            Self::MapJson(_) => PropertyType::Map,
            Self::SetJson(_) => PropertyType::Set,
            Self::Name(_) => PropertyType::Name,
            Self::Text(_) => PropertyType::Text,
            Self::CustomJson(_) => PropertyType::Custom,
            Self::None => PropertyType::Bool, // Default None to bool type
        }
    }
    
    /// Convert to a human-readable string for display/debugging
    pub fn to_string(&self) -> String {
        match self {
            Self::Bool(b) => b.to_string(),
            Self::Byte(b) => b.to_string(),
            Self::Int32(i) => i.to_string(),
            Self::Int64(i) => i.to_string(),
            Self::UInt32(u) => u.to_string(),
            Self::UInt64(u) => u.to_string(),
            Self::Float(f) => f.to_string(),
            Self::Double(d) => d.to_string(),
            Self::String(s) => s.clone(),
            Self::Vector(v) => format!("(X={},Y={},Z={})", v.x, v.y, v.z),
            Self::Rotator(r) => format!("(Pitch={},Yaw={},Roll={})", r.pitch, r.yaw, r.roll),
            Self::Quat(q) => format!("(X={},Y={},Z={},W={})", q.x, q.y, q.z, q.w),
            Self::Transform(t) => format!(
                "Loc:(X={},Y={},Z={}),Rot:(X={},Y={},Z={},W={}),Scale:(X={},Y={},Z={})",
                t.location.x, t.location.y, t.location.z,
                t.rotation.x, t.rotation.y, t.rotation.z, t.rotation.w,
                t.scale.x, t.scale.y, t.scale.z
            ),
            Self::Color(c) => format!("(R={},G={},B={},A={})", c.r, c.g, c.b, c.a),
            Self::ObjectReference(id) => format!("Object:{}", id),
            Self::ClassReference(name) => format!("Class:{}", name),
            Self::ArrayJson(json) => json.clone(),
            Self::MapJson(json) => json.clone(),
            Self::SetJson(json) => json.clone(),
            Self::Name(name) => format!("Name:{}", name),
            Self::Text(text) => format!("Text:{}", text),
            Self::CustomJson(json) => json.clone(),
            Self::None => "None".to_string(),
        }
    }
}

/// Flags for property replication settings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplicationCondition {
    /// Always replicate this property
    Always,
    /// Only replicate when the value changes
    OnChange, 
    /// Only replicate when initial
    Initial,
    /// Only replicate to the owner client
    OwnerOnly,
    /// Only replicate to the server (client to server only)
    ServerOnly,
    /// Custom condition (check via callback, handled in client)
    Custom,
}

/// A property definition with constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyDefinition {
    /// Name of the property
    pub name: String,
    
    /// Type of the property
    pub property_type: PropertyType,
    
    /// Whether the property replicates
    pub replicated: bool,
    
    /// Replication condition (if replicated)
    pub replication_condition: ReplicationCondition,
    
    /// Whether the property is read-only for clients
    pub readonly: bool,
    
    /// Additional flags
    pub flags: u32,
}

impl Default for PropertyDefinition {
    fn default() -> Self {
        Self {
            name: String::new(),
            property_type: PropertyType::Bool,
            replicated: false,
            replication_condition: ReplicationCondition::OnChange,
            readonly: false,
            flags: 0,
        }
    }
} 