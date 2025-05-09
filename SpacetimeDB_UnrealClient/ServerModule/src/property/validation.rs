//! # Property Validation
//!
//! Validation utilities for property values to ensure they conform to expected formats
//! and constraints before being applied to objects.

use crate::property::{PropertyType, PropertyValue};
use std::collections::HashMap;

/// Describes the constraints for a property
#[derive(Debug, Clone)]
pub struct PropertyConstraint {
    /// The type of the property
    pub property_type: PropertyType,
    /// Optional minimum value for numeric types
    pub min_value: Option<PropertyValue>,
    /// Optional maximum value for numeric types
    pub max_value: Option<PropertyValue>,
    /// Optional enum values for string types
    pub allowed_values: Option<Vec<String>>,
    /// Whether the property is required or can be null
    pub required: bool,
    /// Custom validation function name (for client implementation)
    pub custom_validator: Option<String>,
}

/// Validate a property value against its constraints
pub fn validate_property(value: &PropertyValue, constraint: &PropertyConstraint) -> Result<(), String> {
    // Check for null value
    if matches!(value, PropertyValue::None) && constraint.required {
        return Err("Property is required but value is null".to_string());
    }
    
    // Check type compatibility
    let value_type = crate::property::conversion::get_property_type(value);
    if value_type != constraint.property_type && !matches!(value, PropertyValue::None) {
        return Err(format!(
            "Type mismatch: expected {:?}, got {:?}",
            constraint.property_type, value_type
        ));
    }
    
    // Check numeric constraints
    match value {
        PropertyValue::Int32(val) => {
            validate_numeric_constraints(
                *val as f64,
                constraint.min_value.as_ref().map(|v| match v {
                    PropertyValue::Int32(min) => *min as f64,
                    PropertyValue::Float(min) => *min as f64,
                    PropertyValue::Double(min) => *min,
                    _ => f64::MIN,
                }),
                constraint.max_value.as_ref().map(|v| match v {
                    PropertyValue::Int32(max) => *max as f64,
                    PropertyValue::Float(max) => *max as f64,
                    PropertyValue::Double(max) => *max,
                    _ => f64::MAX,
                }),
            )?;
        }
        PropertyValue::Float(val) => {
            validate_numeric_constraints(
                *val as f64,
                constraint.min_value.as_ref().map(|v| match v {
                    PropertyValue::Int32(min) => *min as f64,
                    PropertyValue::Float(min) => *min as f64,
                    PropertyValue::Double(min) => *min,
                    _ => f64::MIN,
                }),
                constraint.max_value.as_ref().map(|v| match v {
                    PropertyValue::Int32(max) => *max as f64,
                    PropertyValue::Float(max) => *max as f64,
                    PropertyValue::Double(max) => *max,
                    _ => f64::MAX,
                }),
            )?;
        }
        PropertyValue::Double(val) => {
            validate_numeric_constraints(
                *val,
                constraint.min_value.as_ref().map(|v| match v {
                    PropertyValue::Int32(min) => *min as f64,
                    PropertyValue::Float(min) => *min as f64,
                    PropertyValue::Double(min) => *min,
                    _ => f64::MIN,
                }),
                constraint.max_value.as_ref().map(|v| match v {
                    PropertyValue::Int32(max) => *max as f64,
                    PropertyValue::Float(max) => *max as f64,
                    PropertyValue::Double(max) => *max,
                    _ => f64::MAX,
                }),
            )?;
        }
        PropertyValue::String(val) => {
            // Check enum constraints
            if let Some(allowed) = &constraint.allowed_values {
                if !allowed.contains(val) {
                    return Err(format!("Value '{}' is not in allowed values: {:?}", val, allowed));
                }
            }
        }
        // For vector/rotator/transform types we could validate min/max for each component
        // but we'll keep it simple for now
        _ => {}
    }
    
    Ok(())
}

/// Validate numeric value against min/max constraints
fn validate_numeric_constraints(
    value: f64,
    min: Option<f64>,
    max: Option<f64>,
) -> Result<(), String> {
    if let Some(min_val) = min {
        if value < min_val {
            return Err(format!("Value {} is less than minimum {}", value, min_val));
        }
    }
    
    if let Some(max_val) = max {
        if value > max_val {
            return Err(format!("Value {} is greater than maximum {}", value, max_val));
        }
    }
    
    Ok(())
}

/// Schema registry for property definitions
#[derive(Debug, Default)]
pub struct PropertySchemaRegistry {
    /// Map of class name to its property definitions
    class_schemas: HashMap<String, ClassPropertySchema>,
}

impl PropertySchemaRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            class_schemas: HashMap::new(),
        }
    }
    
    /// Register a class schema
    pub fn register_class_schema(&mut self, class_name: &str, schema: ClassPropertySchema) {
        self.class_schemas.insert(class_name.to_string(), schema);
    }
    
    /// Get a class schema
    pub fn get_class_schema(&self, class_name: &str) -> Option<&ClassPropertySchema> {
        self.class_schemas.get(class_name)
    }
    
    /// Validate a property value for a specific class and property
    pub fn validate_property(
        &self, 
        class_name: &str, 
        property_name: &str, 
        value: &PropertyValue
    ) -> Result<(), String> {
        // Get class schema
        let schema = self.get_class_schema(class_name)
            .ok_or_else(|| format!("No schema defined for class '{}'", class_name))?;
            
        // Get property constraint
        let constraint = schema.get_property_constraint(property_name)
            .ok_or_else(|| format!("Property '{}' not defined for class '{}'", property_name, class_name))?;
            
        // Validate against constraint
        validate_property(value, constraint)
    }
}

/// Defines the property schema for a class
#[derive(Debug, Clone)]
pub struct ClassPropertySchema {
    /// The name of the class
    pub class_name: String,
    /// The parent class name (if any)
    pub parent_class: Option<String>,
    /// The properties defined for this class
    pub properties: HashMap<String, PropertyConstraint>,
}

impl ClassPropertySchema {
    /// Create a new class property schema
    pub fn new(class_name: &str, parent_class: Option<&str>) -> Self {
        Self {
            class_name: class_name.to_string(),
            parent_class: parent_class.map(|s| s.to_string()),
            properties: HashMap::new(),
        }
    }
    
    /// Add a property constraint
    pub fn add_property(&mut self, name: &str, constraint: PropertyConstraint) {
        self.properties.insert(name.to_string(), constraint);
    }
    
    /// Get a property constraint
    pub fn get_property_constraint(&self, name: &str) -> Option<&PropertyConstraint> {
        self.properties.get(name)
    }
} 