//! # Property Serialization
//!
//! Utilities for serializing and deserializing PropertyValue to/from JSON and
//! other formats for network transmission and storage.

use crate::property::{PropertyType, PropertyValue};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// JSON-serializable representation of a PropertyValue
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SerializedPropertyValue {
    /// The type of the property
    #[serde(rename = "type")]
    pub property_type: String,
    
    /// The value, serialized as JSON
    pub value: JsonValue,
}

/// Serialize a PropertyValue to JSON
pub fn serialize_property_value(value: &PropertyValue) -> Result<String, String> {
    let serialized = match value {
        PropertyValue::Bool(b) => SerializedPropertyValue {
            property_type: "Bool".to_string(),
            value: JsonValue::Bool(*b),
        },
        PropertyValue::Byte(b) => SerializedPropertyValue {
            property_type: "Byte".to_string(),
            value: JsonValue::Number((*b).into()),
        },
        PropertyValue::Int32(i) => SerializedPropertyValue {
            property_type: "Int32".to_string(),
            value: JsonValue::Number((*i).into()),
        },
        PropertyValue::Int64(i) => SerializedPropertyValue {
            property_type: "Int64".to_string(),
            value: JsonValue::Number((*i).into()),
        },
        PropertyValue::UInt32(u) => SerializedPropertyValue {
            property_type: "UInt32".to_string(),
            value: JsonValue::Number((*u).into()),
        },
        PropertyValue::UInt64(u) => SerializedPropertyValue {
            property_type: "UInt64".to_string(),
            // JSON doesn't support u64 directly, so convert to string
            value: JsonValue::String(u.to_string()),
        },
        PropertyValue::Float(f) => SerializedPropertyValue {
            property_type: "Float".to_string(),
            value: JsonValue::Number(serde_json::Number::from_f64(*f as f64).unwrap_or_default()),
        },
        PropertyValue::Double(d) => SerializedPropertyValue {
            property_type: "Double".to_string(),
            value: JsonValue::Number(serde_json::Number::from_f64(*d).unwrap_or_default()),
        },
        PropertyValue::String(s) => SerializedPropertyValue {
            property_type: "String".to_string(),
            value: JsonValue::String(s.clone()),
        },
        PropertyValue::Vector { x, y, z } => {
            let mut map = serde_json::Map::new();
            map.insert("x".to_string(), JsonValue::Number((*x as f64).into()));
            map.insert("y".to_string(), JsonValue::Number((*y as f64).into()));
            map.insert("z".to_string(), JsonValue::Number((*z as f64).into()));
            
            SerializedPropertyValue {
                property_type: "Vector".to_string(),
                value: JsonValue::Object(map),
            }
        },
        PropertyValue::Rotator { pitch, yaw, roll } => {
            let mut map = serde_json::Map::new();
            map.insert("pitch".to_string(), JsonValue::Number((*pitch as f64).into()));
            map.insert("yaw".to_string(), JsonValue::Number((*yaw as f64).into()));
            map.insert("roll".to_string(), JsonValue::Number((*roll as f64).into()));
            
            SerializedPropertyValue {
                property_type: "Rotator".to_string(),
                value: JsonValue::Object(map),
            }
        },
        PropertyValue::Quat { x, y, z, w } => {
            let mut map = serde_json::Map::new();
            map.insert("x".to_string(), JsonValue::Number((*x as f64).into()));
            map.insert("y".to_string(), JsonValue::Number((*y as f64).into()));
            map.insert("z".to_string(), JsonValue::Number((*z as f64).into()));
            map.insert("w".to_string(), JsonValue::Number((*w as f64).into()));
            
            SerializedPropertyValue {
                property_type: "Quat".to_string(),
                value: JsonValue::Object(map),
            }
        },
        PropertyValue::Transform { 
            pos_x, pos_y, pos_z, 
            rot_x, rot_y, rot_z, rot_w, 
            scale_x, scale_y, scale_z 
        } => {
            let mut position = serde_json::Map::new();
            position.insert("x".to_string(), JsonValue::Number((*pos_x as f64).into()));
            position.insert("y".to_string(), JsonValue::Number((*pos_y as f64).into()));
            position.insert("z".to_string(), JsonValue::Number((*pos_z as f64).into()));
            
            let mut rotation = serde_json::Map::new();
            rotation.insert("x".to_string(), JsonValue::Number((*rot_x as f64).into()));
            rotation.insert("y".to_string(), JsonValue::Number((*rot_y as f64).into()));
            rotation.insert("z".to_string(), JsonValue::Number((*rot_z as f64).into()));
            rotation.insert("w".to_string(), JsonValue::Number((*rot_w as f64).into()));
            
            let mut scale = serde_json::Map::new();
            scale.insert("x".to_string(), JsonValue::Number((*scale_x as f64).into()));
            scale.insert("y".to_string(), JsonValue::Number((*scale_y as f64).into()));
            scale.insert("z".to_string(), JsonValue::Number((*scale_z as f64).into()));
            
            let mut map = serde_json::Map::new();
            map.insert("position".to_string(), JsonValue::Object(position));
            map.insert("rotation".to_string(), JsonValue::Object(rotation));
            map.insert("scale".to_string(), JsonValue::Object(scale));
            
            SerializedPropertyValue {
                property_type: "Transform".to_string(),
                value: JsonValue::Object(map),
            }
        },
        PropertyValue::Color { r, g, b, a } => {
            let mut map = serde_json::Map::new();
            map.insert("r".to_string(), JsonValue::Number((*r).into()));
            map.insert("g".to_string(), JsonValue::Number((*g).into()));
            map.insert("b".to_string(), JsonValue::Number((*b).into()));
            map.insert("a".to_string(), JsonValue::Number((*a).into()));
            
            SerializedPropertyValue {
                property_type: "Color".to_string(),
                value: JsonValue::Object(map),
            }
        },
        PropertyValue::ObjectReference(id) => SerializedPropertyValue {
            property_type: "ObjectReference".to_string(),
            value: JsonValue::Number((*id as u64).into()),
        },
        PropertyValue::ClassReference(class_name) => SerializedPropertyValue {
            property_type: "ClassReference".to_string(),
            value: JsonValue::String(class_name.clone()),
        },
        PropertyValue::ArrayJson(json) => {
            let parsed: JsonValue = serde_json::from_str(json)
                .map_err(|e| format!("Failed to parse array JSON: {}", e))?;
                
            SerializedPropertyValue {
                property_type: "Array".to_string(),
                value: parsed,
            }
        },
        PropertyValue::MapJson(json) => {
            let parsed: JsonValue = serde_json::from_str(json)
                .map_err(|e| format!("Failed to parse map JSON: {}", e))?;
                
            SerializedPropertyValue {
                property_type: "Map".to_string(),
                value: parsed,
            }
        },
        PropertyValue::SetJson(json) => {
            let parsed: JsonValue = serde_json::from_str(json)
                .map_err(|e| format!("Failed to parse set JSON: {}", e))?;
                
            SerializedPropertyValue {
                property_type: "Set".to_string(),
                value: parsed,
            }
        },
        PropertyValue::Name(name) => SerializedPropertyValue {
            property_type: "Name".to_string(),
            value: JsonValue::String(name.clone()),
        },
        PropertyValue::Text(text) => SerializedPropertyValue {
            property_type: "Text".to_string(),
            value: JsonValue::String(text.clone()),
        },
        PropertyValue::CustomJson(json) => {
            let parsed: JsonValue = serde_json::from_str(json)
                .map_err(|e| format!("Failed to parse custom JSON: {}", e))?;
                
            SerializedPropertyValue {
                property_type: "Custom".to_string(),
                value: parsed,
            }
        },
        PropertyValue::None => SerializedPropertyValue {
            property_type: "None".to_string(),
            value: JsonValue::Null,
        },
    };
    
    serde_json::to_string(&serialized)
        .map_err(|e| format!("Failed to serialize property value: {}", e))
}

/// Deserialize a PropertyValue from JSON
pub fn deserialize_property_value(json: &str) -> Result<PropertyValue, String> {
    let serialized: SerializedPropertyValue = serde_json::from_str(json)
        .map_err(|e| format!("Failed to deserialize property value: {}", e))?;
        
    match serialized.property_type.as_str() {
        "Bool" => {
            let value = serialized.value.as_bool()
                .ok_or_else(|| "Expected boolean value".to_string())?;
            Ok(PropertyValue::Bool(value))
        },
        "Byte" => {
            let value = serialized.value.as_u64()
                .ok_or_else(|| "Expected numeric value".to_string())?;
            Ok(PropertyValue::Byte(value as u8))
        },
        "Int32" => {
            let value = serialized.value.as_i64()
                .ok_or_else(|| "Expected numeric value".to_string())?;
            Ok(PropertyValue::Int32(value as i32))
        },
        "Int64" => {
            let value = serialized.value.as_i64()
                .ok_or_else(|| "Expected numeric value".to_string())?;
            Ok(PropertyValue::Int64(value))
        },
        "UInt32" => {
            let value = serialized.value.as_u64()
                .ok_or_else(|| "Expected numeric value".to_string())?;
            Ok(PropertyValue::UInt32(value as u32))
        },
        "UInt64" => {
            let value = if let Some(num) = serialized.value.as_u64() {
                num
            } else if let Some(str_val) = serialized.value.as_str() {
                str_val.parse::<u64>()
                    .map_err(|e| format!("Failed to parse u64: {}", e))?
            } else {
                return Err("Expected numeric or string value for UInt64".to_string());
            };
            Ok(PropertyValue::UInt64(value))
        },
        "Float" => {
            let value = serialized.value.as_f64()
                .ok_or_else(|| "Expected floating point value".to_string())?;
            Ok(PropertyValue::Float(value as f32))
        },
        "Double" => {
            let value = serialized.value.as_f64()
                .ok_or_else(|| "Expected floating point value".to_string())?;
            Ok(PropertyValue::Double(value))
        },
        "String" => {
            let value = serialized.value.as_str()
                .ok_or_else(|| "Expected string value".to_string())?;
            Ok(PropertyValue::String(value.to_string()))
        },
        "Vector" => {
            let obj = serialized.value.as_object()
                .ok_or_else(|| "Expected object for Vector".to_string())?;
                
            let x = obj.get("x").and_then(|v| v.as_f64())
                .ok_or_else(|| "Missing x component".to_string())? as f32;
            let y = obj.get("y").and_then(|v| v.as_f64())
                .ok_or_else(|| "Missing y component".to_string())? as f32;
            let z = obj.get("z").and_then(|v| v.as_f64())
                .ok_or_else(|| "Missing z component".to_string())? as f32;
                
            Ok(PropertyValue::Vector { x, y, z })
        },
        "Rotator" => {
            let obj = serialized.value.as_object()
                .ok_or_else(|| "Expected object for Rotator".to_string())?;
                
            let pitch = obj.get("pitch").and_then(|v| v.as_f64())
                .ok_or_else(|| "Missing pitch component".to_string())? as f32;
            let yaw = obj.get("yaw").and_then(|v| v.as_f64())
                .ok_or_else(|| "Missing yaw component".to_string())? as f32;
            let roll = obj.get("roll").and_then(|v| v.as_f64())
                .ok_or_else(|| "Missing roll component".to_string())? as f32;
                
            Ok(PropertyValue::Rotator { pitch, yaw, roll })
        },
        "Quat" => {
            let obj = serialized.value.as_object()
                .ok_or_else(|| "Expected object for Quat".to_string())?;
                
            let x = obj.get("x").and_then(|v| v.as_f64())
                .ok_or_else(|| "Missing x component".to_string())? as f32;
            let y = obj.get("y").and_then(|v| v.as_f64())
                .ok_or_else(|| "Missing y component".to_string())? as f32;
            let z = obj.get("z").and_then(|v| v.as_f64())
                .ok_or_else(|| "Missing z component".to_string())? as f32;
            let w = obj.get("w").and_then(|v| v.as_f64())
                .ok_or_else(|| "Missing w component".to_string())? as f32;
                
            Ok(PropertyValue::Quat { x, y, z, w })
        },
        "Transform" => {
            let obj = serialized.value.as_object()
                .ok_or_else(|| "Expected object for Transform".to_string())?;
                
            let position = obj.get("position").and_then(|v| v.as_object())
                .ok_or_else(|| "Missing position".to_string())?;
            let rotation = obj.get("rotation").and_then(|v| v.as_object())
                .ok_or_else(|| "Missing rotation".to_string())?;
            let scale = obj.get("scale").and_then(|v| v.as_object())
                .ok_or_else(|| "Missing scale".to_string())?;
                
            let pos_x = position.get("x").and_then(|v| v.as_f64())
                .ok_or_else(|| "Missing position.x".to_string())? as f32;
            let pos_y = position.get("y").and_then(|v| v.as_f64())
                .ok_or_else(|| "Missing position.y".to_string())? as f32;
            let pos_z = position.get("z").and_then(|v| v.as_f64())
                .ok_or_else(|| "Missing position.z".to_string())? as f32;
                
            let rot_x = rotation.get("x").and_then(|v| v.as_f64())
                .ok_or_else(|| "Missing rotation.x".to_string())? as f32;
            let rot_y = rotation.get("y").and_then(|v| v.as_f64())
                .ok_or_else(|| "Missing rotation.y".to_string())? as f32;
            let rot_z = rotation.get("z").and_then(|v| v.as_f64())
                .ok_or_else(|| "Missing rotation.z".to_string())? as f32;
            let rot_w = rotation.get("w").and_then(|v| v.as_f64())
                .ok_or_else(|| "Missing rotation.w".to_string())? as f32;
                
            let scale_x = scale.get("x").and_then(|v| v.as_f64())
                .ok_or_else(|| "Missing scale.x".to_string())? as f32;
            let scale_y = scale.get("y").and_then(|v| v.as_f64())
                .ok_or_else(|| "Missing scale.y".to_string())? as f32;
            let scale_z = scale.get("z").and_then(|v| v.as_f64())
                .ok_or_else(|| "Missing scale.z".to_string())? as f32;
                
            Ok(PropertyValue::Transform {
                pos_x, pos_y, pos_z,
                rot_x, rot_y, rot_z, rot_w,
                scale_x, scale_y, scale_z,
            })
        },
        "Color" => {
            let obj = serialized.value.as_object()
                .ok_or_else(|| "Expected object for Color".to_string())?;
                
            let r = obj.get("r").and_then(|v| v.as_u64())
                .ok_or_else(|| "Missing r component".to_string())? as u8;
            let g = obj.get("g").and_then(|v| v.as_u64())
                .ok_or_else(|| "Missing g component".to_string())? as u8;
            let b = obj.get("b").and_then(|v| v.as_u64())
                .ok_or_else(|| "Missing b component".to_string())? as u8;
            let a = obj.get("a").and_then(|v| v.as_u64())
                .ok_or_else(|| "Missing a component".to_string())? as u8;
                
            Ok(PropertyValue::Color { r, g, b, a })
        },
        "ObjectReference" => {
            let value = serialized.value.as_u64()
                .ok_or_else(|| "Expected numeric value for ObjectReference".to_string())?;
            Ok(PropertyValue::ObjectReference(value as u64))
        },
        "ClassReference" => {
            let value = serialized.value.as_str()
                .ok_or_else(|| "Expected string value for ClassReference".to_string())?;
            Ok(PropertyValue::ClassReference(value.to_string()))
        },
        "Array" => {
            let json = serde_json::to_string(&serialized.value)
                .map_err(|e| format!("Failed to serialize array: {}", e))?;
            Ok(PropertyValue::ArrayJson(json))
        },
        "Map" => {
            let json = serde_json::to_string(&serialized.value)
                .map_err(|e| format!("Failed to serialize map: {}", e))?;
            Ok(PropertyValue::MapJson(json))
        },
        "Set" => {
            let json = serde_json::to_string(&serialized.value)
                .map_err(|e| format!("Failed to serialize set: {}", e))?;
            Ok(PropertyValue::SetJson(json))
        },
        "Name" => {
            let value = serialized.value.as_str()
                .ok_or_else(|| "Expected string value for Name".to_string())?;
            Ok(PropertyValue::Name(value.to_string()))
        },
        "Text" => {
            let value = serialized.value.as_str()
                .ok_or_else(|| "Expected string value for Text".to_string())?;
            Ok(PropertyValue::Text(value.to_string()))
        },
        "Custom" => {
            let json = serde_json::to_string(&serialized.value)
                .map_err(|e| format!("Failed to serialize custom: {}", e))?;
            Ok(PropertyValue::CustomJson(json))
        },
        "None" => Ok(PropertyValue::None),
        _ => Err(format!("Unknown property type: {}", serialized.property_type)),
    }
}

/// Serialize a map of property values to JSON
pub fn serialize_property_map(properties: &HashMap<String, PropertyValue>) -> Result<String, String> {
    let mut result = HashMap::new();
    
    for (name, value) in properties {
        let serialized = serialize_property_value(value)?;
        let parsed: SerializedPropertyValue = serde_json::from_str(&serialized)
            .map_err(|e| format!("Failed to parse serialized property: {}", e))?;
            
        result.insert(name.clone(), parsed);
    }
    
    serde_json::to_string(&result)
        .map_err(|e| format!("Failed to serialize property map: {}", e))
}

/// Deserialize a map of property values from JSON
pub fn deserialize_property_map(json: &str) -> Result<HashMap<String, PropertyValue>, String> {
    let parsed: HashMap<String, SerializedPropertyValue> = serde_json::from_str(json)
        .map_err(|e| format!("Failed to deserialize property map: {}", e))?;
        
    let mut result = HashMap::new();
    
    for (name, serialized) in parsed {
        let json = serde_json::to_string(&serialized)
            .map_err(|e| format!("Failed to serialize property: {}", e))?;
            
        let value = deserialize_property_value(&json)?;
        result.insert(name, value);
    }
    
    Ok(result)
} 