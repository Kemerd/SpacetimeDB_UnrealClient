//! # Property Serialization
//!
//! Provides serialization and deserialization utilities for property values,
//! allowing conversion between PropertyValue enum and JSON representations.

use stdb_shared::property::{PropertyType, PropertyValue};
use stdb_shared::types::*;
use stdb_shared::object::ObjectId;
use serde_json::{Value, json};
use std::collections::HashMap;
use log::{warn};

/// Serialize a property value to a JSON string
pub fn serialize_property_value(value: &PropertyValue) -> Result<String, String> {
    let json_value = match value {
        // Primitive values
        PropertyValue::Bool(b) => json!(*b),
        PropertyValue::Byte(b) => json!(*b),
        PropertyValue::Int32(i) => json!(*i),
        PropertyValue::Int64(i) => json!(*i),
        PropertyValue::UInt32(u) => json!(*u),
        PropertyValue::UInt64(u) => json!(*u),
        PropertyValue::Float(f) => json!(*f),
        PropertyValue::Double(d) => json!(*d),
        PropertyValue::String(s) => json!(s),
        
        // Structured values
        PropertyValue::Vector(v) => json!({
            "x": v.x,
            "y": v.y,
            "z": v.z
        }),
        PropertyValue::Rotator(r) => json!({
            "pitch": r.pitch,
            "yaw": r.yaw,
            "roll": r.roll
        }),
        PropertyValue::Quat(q) => json!({
            "x": q.x,
            "y": q.y,
            "z": q.z,
            "w": q.w
        }),
        PropertyValue::Transform(t) => json!({
            "location": {
                "x": t.location.x,
                "y": t.location.y,
                "z": t.location.z
            },
            "rotation": {
                "x": t.rotation.x,
                "y": t.rotation.y,
                "z": t.rotation.z,
                "w": t.rotation.w
            },
            "scale": {
                "x": t.scale.x,
                "y": t.scale.y,
                "z": t.scale.z
            }
        }),
        PropertyValue::Color(c) => json!({
            "r": c.r,
            "g": c.g,
            "b": c.b,
            "a": c.a
        }),
        
        // Reference values
        PropertyValue::ObjectReference(id) => json!({
            "type": "ObjectReference",
            "id": *id
        }),
        PropertyValue::ClassReference(name) => json!({
            "type": "ClassReference",
            "name": name
        }),
        
        // Container values (already stored as JSON strings)
        PropertyValue::ArrayJson(json_str) => 
            serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON in ArrayJson: {}", e))?,
        PropertyValue::MapJson(json_str) => 
            serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON in MapJson: {}", e))?,
        PropertyValue::SetJson(json_str) => 
            serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON in SetJson: {}", e))?,
        
        // Special values
        PropertyValue::Name(name) => json!({
            "type": "Name",
            "value": name
        }),
        PropertyValue::Text(text) => json!({
            "type": "Text",
            "value": text
        }),
        PropertyValue::CustomJson(json_str) => 
            serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON in CustomJson: {}", e))?,
        
        // Null
        PropertyValue::None => Value::Null,
    };
    
    serde_json::to_string(&json_value)
        .map_err(|e| format!("Failed to serialize PropertyValue to JSON: {}", e))
}

/// Deserialize a property value from a JSON string
pub fn deserialize_property_value(json_str: &str) -> Result<PropertyValue, String> {
    let json_value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Invalid JSON: {}", e))?;
    
    // First try to determine if this is a tagged type with explicit type information
    if let Value::Object(map) = &json_value {
        if let Some(Value::String(type_name)) = map.get("type") {
            match type_name.as_str() {
                "ObjectReference" => {
                    if let Some(Value::Number(id)) = map.get("id") {
                        if let Some(id) = id.as_u64() {
                            return Ok(PropertyValue::ObjectReference(id));
                        }
                    }
                    return Err("Invalid ObjectReference format".to_string());
                },
                "ClassReference" => {
                    if let Some(Value::String(name)) = map.get("name") {
                        return Ok(PropertyValue::ClassReference(name.clone()));
                    }
                    return Err("Invalid ClassReference format".to_string());
                },
                "Name" => {
                    if let Some(Value::String(value)) = map.get("value") {
                        return Ok(PropertyValue::Name(value.clone()));
                    }
                    return Err("Invalid Name format".to_string());
                },
                "Text" => {
                    if let Some(Value::String(value)) = map.get("value") {
                        return Ok(PropertyValue::Text(value.clone()));
                    }
                    return Err("Invalid Text format".to_string());
                },
                _ => {
                    warn!("Unknown type tag '{}' in JSON", type_name);
                    // Continue with structure-based detection
                }
            }
        }
        
        // Structure detection for common types without explicit tags
        if map.contains_key("x") && map.contains_key("y") && map.contains_key("z") {
            if map.contains_key("w") {
                // Likely a quaternion
                let x = parse_f32_field(map, "x")?;
                let y = parse_f32_field(map, "y")?;
                let z = parse_f32_field(map, "z")?;
                let w = parse_f32_field(map, "w")?;
                return Ok(PropertyValue::Quat(Quat { x, y, z, w }));
            } else {
                // Likely a vector
                let x = parse_f32_field(map, "x")?;
                let y = parse_f32_field(map, "y")?;
                let z = parse_f32_field(map, "z")?;
                return Ok(PropertyValue::Vector(Vector3 { x, y, z }));
            }
        }
        
        if map.contains_key("pitch") && map.contains_key("yaw") && map.contains_key("roll") {
            // Likely a rotator
            let pitch = parse_f32_field(map, "pitch")?;
            let yaw = parse_f32_field(map, "yaw")?;
            let roll = parse_f32_field(map, "roll")?;
            return Ok(PropertyValue::Rotator(Rotator { pitch, yaw, roll }));
        }
        
        if map.contains_key("r") && map.contains_key("g") && map.contains_key("b") {
            // Likely a color
            let r = parse_u8_field(map, "r")?;
            let g = parse_u8_field(map, "g")?;
            let b = parse_u8_field(map, "b")?;
            let a = map.get("a")
                .and_then(|v| v.as_u64())
                .map(|v| v as u8)
                .unwrap_or(255);
            return Ok(PropertyValue::Color(Color { r, g, b, a }));
        }
        
        if map.contains_key("location") && map.contains_key("rotation") && map.contains_key("scale") {
            // Likely a transform
            let location = parse_vector3(map.get("location"))?;
            let rotation = parse_quat(map.get("rotation"))?;
            let scale = parse_vector3(map.get("scale"))?;
            
            return Ok(PropertyValue::Transform(Transform {
                location,
                rotation,
                scale,
            }));
        }
        
        // If no structural match, serialize back to JSON for container types
        let json_string = serde_json::to_string(&json_value)
            .map_err(|e| format!("Failed to re-serialize JSON: {}", e))?;
        
        return Ok(PropertyValue::CustomJson(json_string));
    }
    
    // Handle primitive types
    match &json_value {
        Value::Null => Ok(PropertyValue::None),
        Value::Bool(b) => Ok(PropertyValue::Bool(*b)),
        Value::Number(n) => {
            if n.is_i64() {
                if let Some(i) = n.as_i64() {
                    if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
                        Ok(PropertyValue::Int32(i as i32))
                    } else {
                        Ok(PropertyValue::Int64(i))
                    }
                } else {
                    Err("Failed to convert number to i64".to_string())
                }
            } else if n.is_u64() {
                if let Some(u) = n.as_u64() {
                    if u <= u32::MAX as u64 {
                        Ok(PropertyValue::UInt32(u as u32))
                    } else {
                        Ok(PropertyValue::UInt64(u))
                    }
                } else {
                    Err("Failed to convert number to u64".to_string())
                }
            } else if n.is_f64() {
                if let Some(f) = n.as_f64() {
                    if f >= f32::MIN as f64 && f <= f32::MAX as f64 {
                        Ok(PropertyValue::Float(f as f32))
                    } else {
                        Ok(PropertyValue::Double(f))
                    }
                } else {
                    Err("Failed to convert number to f64".to_string())
                }
            } else {
                Err("Unsupported number format".to_string())
            }
        },
        Value::String(s) => Ok(PropertyValue::String(s.clone())),
        Value::Array(_) => {
            let json_string = serde_json::to_string(&json_value)
                .map_err(|e| format!("Failed to serialize array: {}", e))?;
            Ok(PropertyValue::ArrayJson(json_string))
        },
        Value::Object(_) => {
            let json_string = serde_json::to_string(&json_value)
                .map_err(|e| format!("Failed to serialize object: {}", e))?;
            Ok(PropertyValue::MapJson(json_string))
        }
    }
}

// Helper functions for parsing structured types

fn parse_f32_field(map: &serde_json::Map<String, Value>, field: &str) -> Result<f32, String> {
    match map.get(field) {
        Some(Value::Number(n)) => {
            n.as_f64()
                .map(|f| f as f32)
                .ok_or_else(|| format!("Invalid number for field '{}'", field))
        },
        _ => Err(format!("Missing or invalid field '{}'", field)),
    }
}

fn parse_u8_field(map: &serde_json::Map<String, Value>, field: &str) -> Result<u8, String> {
    match map.get(field) {
        Some(Value::Number(n)) => {
            n.as_u64()
                .map(|u| u as u8)
                .ok_or_else(|| format!("Invalid number for field '{}'", field))
        },
        _ => Err(format!("Missing or invalid field '{}'", field)),
    }
}

fn parse_vector3(value: Option<&Value>) -> Result<Vector3, String> {
    match value {
        Some(Value::Object(map)) => {
            let x = parse_f32_field(map, "x")?;
            let y = parse_f32_field(map, "y")?;
            let z = parse_f32_field(map, "z")?;
            Ok(Vector3 { x, y, z })
        },
        _ => Err("Missing or invalid Vector3 structure".to_string()),
    }
}

fn parse_quat(value: Option<&Value>) -> Result<Quat, String> {
    match value {
        Some(Value::Object(map)) => {
            let x = parse_f32_field(map, "x")?;
            let y = parse_f32_field(map, "y")?;
            let z = parse_f32_field(map, "z")?;
            let w = parse_f32_field(map, "w")?;
            Ok(Quat { x, y, z, w })
        },
        _ => Err("Missing or invalid Quat structure".to_string()),
    }
} 