//! # Property Conversion
//!
//! Utility functions for converting between different property types and formats.
//! This helps handle the translation between Unreal Engine types and SpacetimeDB.

use crate::property::{PropertyType, PropertyValue};
use std::str::FromStr;

/// Converts a string representation to a PropertyValue
/// For simplicity in the API, this allows passing values as strings that get parsed
pub fn string_to_property_value(value_str: &str) -> PropertyValue {
    // Simple heuristic - try to determine type from string content
    // In a real implementation, you'd know the expected type
    
    // Empty string = None
    if value_str.is_empty() {
        return PropertyValue::None;
    }
    
    // Try bool
    if value_str == "true" {
        return PropertyValue::Bool(true);
    }
    if value_str == "false" {
        return PropertyValue::Bool(false);
    }
    
    // Try integer
    if let Ok(int_val) = i32::from_str(value_str) {
        return PropertyValue::Int32(int_val);
    }
    
    // Try float
    if let Ok(float_val) = f32::from_str(value_str) {
        return PropertyValue::Float(float_val);
    }
    
    // Check for vector format "(X=1.0,Y=2.0,Z=3.0)"
    if value_str.starts_with('(') && value_str.ends_with(')') && value_str.contains("X=") && value_str.contains("Y=") && value_str.contains("Z=") {
        return parse_vector(value_str);
    }
    
    // Check for rotator format "(Pitch=1.0,Yaw=2.0,Roll=3.0)"
    if value_str.starts_with('(') && value_str.ends_with(')') && value_str.contains("Pitch=") && value_str.contains("Yaw=") && value_str.contains("Roll=") {
        return parse_rotator(value_str);
    }
    
    // Check for color format "(R=255,G=128,B=64,A=255)"
    if value_str.starts_with('(') && value_str.ends_with(')') && value_str.contains("R=") && value_str.contains("G=") && value_str.contains("B=") {
        return parse_color(value_str);
    }
    
    // Default to string
    PropertyValue::String(value_str.to_string())
}

/// Attempts to parse a vector from a string like "(X=1.0,Y=2.0,Z=3.0)"
fn parse_vector(value_str: &str) -> PropertyValue {
    // Default values
    let mut x = 0.0;
    let mut y = 0.0;
    let mut z = 0.0;
    
    // Strip parentheses and split by comma
    let content = &value_str[1..value_str.len()-1];
    for part in content.split(',') {
        let part = part.trim();
        
        // Extract component and value
        if let Some(eq_pos) = part.find('=') {
            let (component, val_str) = part.split_at(eq_pos);
            let val_str = &val_str[1..]; // Skip the '=' character
            
            // Parse value
            if let Ok(val) = f32::from_str(val_str) {
                match component.trim() {
                    "X" => x = val,
                    "Y" => y = val,
                    "Z" => z = val,
                    _ => {}
                }
            }
        }
    }
    
    PropertyValue::Vector { x, y, z }
}

/// Attempts to parse a rotator from a string like "(Pitch=1.0,Yaw=2.0,Roll=3.0)"
fn parse_rotator(value_str: &str) -> PropertyValue {
    // Default values
    let mut pitch = 0.0;
    let mut yaw = 0.0;
    let mut roll = 0.0;
    
    // Strip parentheses and split by comma
    let content = &value_str[1..value_str.len()-1];
    for part in content.split(',') {
        let part = part.trim();
        
        // Extract component and value
        if let Some(eq_pos) = part.find('=') {
            let (component, val_str) = part.split_at(eq_pos);
            let val_str = &val_str[1..]; // Skip the '=' character
            
            // Parse value
            if let Ok(val) = f32::from_str(val_str) {
                match component.trim() {
                    "Pitch" => pitch = val,
                    "Yaw" => yaw = val,
                    "Roll" => roll = val,
                    _ => {}
                }
            }
        }
    }
    
    PropertyValue::Rotator { pitch, yaw, roll }
}

/// Attempts to parse a color from a string like "(R=255,G=128,B=64,A=255)"
fn parse_color(value_str: &str) -> PropertyValue {
    // Default values
    let mut r = 0;
    let mut g = 0;
    let mut b = 0;
    let mut a = 255; // Default alpha to fully opaque
    
    // Strip parentheses and split by comma
    let content = &value_str[1..value_str.len()-1];
    for part in content.split(',') {
        let part = part.trim();
        
        // Extract component and value
        if let Some(eq_pos) = part.find('=') {
            let (component, val_str) = part.split_at(eq_pos);
            let val_str = &val_str[1..]; // Skip the '=' character
            
            // Parse value
            if let Ok(val) = u8::from_str(val_str) {
                match component.trim() {
                    "R" => r = val,
                    "G" => g = val,
                    "B" => b = val,
                    "A" => a = val,
                    _ => {}
                }
            }
        }
    }
    
    PropertyValue::Color { r, g, b, a }
}

/// Gets the PropertyType of a PropertyValue
pub fn get_property_type(value: &PropertyValue) -> PropertyType {
    match value {
        PropertyValue::Bool(_) => PropertyType::Bool,
        PropertyValue::Byte(_) => PropertyType::Byte,
        PropertyValue::Int32(_) => PropertyType::Int32,
        PropertyValue::Int64(_) => PropertyType::Int64,
        PropertyValue::UInt32(_) => PropertyType::UInt32,
        PropertyValue::UInt64(_) => PropertyType::UInt64,
        PropertyValue::Float(_) => PropertyType::Float,
        PropertyValue::Double(_) => PropertyType::Double,
        PropertyValue::String(_) => PropertyType::String,
        PropertyValue::Vector { .. } => PropertyType::Vector,
        PropertyValue::Rotator { .. } => PropertyType::Rotator,
        PropertyValue::Quat { .. } => PropertyType::Quat,
        PropertyValue::Transform { .. } => PropertyType::Transform,
        PropertyValue::Color { .. } => PropertyType::Color,
        PropertyValue::ObjectReference(_) => PropertyType::ObjectReference,
        PropertyValue::ClassReference(_) => PropertyType::ClassReference,
        PropertyValue::ArrayJson(_) => PropertyType::Array,
        PropertyValue::MapJson(_) => PropertyType::Map,
        PropertyValue::SetJson(_) => PropertyType::Set,
        PropertyValue::Name(_) => PropertyType::Name,
        PropertyValue::Text(_) => PropertyType::Text,
        PropertyValue::CustomJson(_) => PropertyType::Custom,
        PropertyValue::None => PropertyType::Bool, // Default None to bool type
    }
}

/// Convert PropertyValue to string representation (for debugging/display)
pub fn property_value_to_string(value: &PropertyValue) -> String {
    match value {
        PropertyValue::Bool(b) => b.to_string(),
        PropertyValue::Byte(b) => b.to_string(),
        PropertyValue::Int32(i) => i.to_string(),
        PropertyValue::Int64(i) => i.to_string(),
        PropertyValue::UInt32(u) => u.to_string(),
        PropertyValue::UInt64(u) => u.to_string(),
        PropertyValue::Float(f) => f.to_string(),
        PropertyValue::Double(d) => d.to_string(),
        PropertyValue::String(s) => s.clone(),
        PropertyValue::Vector { x, y, z } => format!("(X={},Y={},Z={})", x, y, z),
        PropertyValue::Rotator { pitch, yaw, roll } => format!("(Pitch={},Yaw={},Roll={})", pitch, yaw, roll),
        PropertyValue::Quat { x, y, z, w } => format!("(X={},Y={},Z={},W={})", x, y, z, w),
        PropertyValue::Transform { pos_x, pos_y, pos_z, rot_x, rot_y, rot_z, rot_w, scale_x, scale_y, scale_z } => 
            format!("Pos:(X={},Y={},Z={}),Rot:(X={},Y={},Z={},W={}),Scale:(X={},Y={},Z={})", 
                    pos_x, pos_y, pos_z, rot_x, rot_y, rot_z, rot_w, scale_x, scale_y, scale_z),
        PropertyValue::Color { r, g, b, a } => format!("(R={},G={},B={},A={})", r, g, b, a),
        PropertyValue::ObjectReference(id) => format!("Object:{}", id),
        PropertyValue::ClassReference(id) => format!("Class:{}", id),
        PropertyValue::ArrayJson(json) => json.clone(),
        PropertyValue::MapJson(json) => json.clone(),
        PropertyValue::SetJson(json) => json.clone(),
        PropertyValue::Name(name) => format!("Name:{}", name),
        PropertyValue::Text(text) => format!("Text:{}", text),
        PropertyValue::CustomJson(json) => json.clone(),
        PropertyValue::None => "None".to_string(),
    }
} 