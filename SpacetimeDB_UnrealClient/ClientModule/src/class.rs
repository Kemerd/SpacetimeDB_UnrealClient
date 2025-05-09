//! # Class System
//! 
//! This module provides functionality for working with classes in the system.
//! It handles class IDs, actor class detection, and other class-related operations.

use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use log::{debug, info};

/// Map of class names to class IDs
static CLASS_ID_MAP: Lazy<Mutex<HashMap<String, u32>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Map of class IDs to information about whether it's an actor class
static ACTOR_CLASS_MAP: Lazy<Mutex<HashMap<u32, bool>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Map of class names to parent class names
static CLASS_HIERARCHY: Lazy<Mutex<HashMap<String, String>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Create a class with the given parent class
pub fn create_class(class_name: &str, parent_class_name: &str) -> bool {
    // Check if class already exists
    {
        let class_map = CLASS_ID_MAP.lock().unwrap();
        if class_map.contains_key(class_name) {
            return false; // Class already exists
        }
    }
    
    // Generate a class ID (simple hash of the name for this example)
    let class_id = generate_class_id(class_name);
    
    // Determine if this is an actor class based on parent
    let is_actor = if parent_class_name.is_empty() {
        // If no parent, assume it's not an actor
        false
    } else {
        // If parent is "Actor" or any class that is an actor, then this is an actor too
        parent_class_name == "Actor" || 
            get_class_id_by_name(parent_class_name)
                .map(|id| is_actor_class(id))
                .unwrap_or(false)
    };
    
    // Register the class
    {
        let mut class_map = CLASS_ID_MAP.lock().unwrap();
        class_map.insert(class_name.to_string(), class_id);
    }
    
    // Register the actor status
    {
        let mut actor_map = ACTOR_CLASS_MAP.lock().unwrap();
        actor_map.insert(class_id, is_actor);
    }
    
    // Register the parent class
    if !parent_class_name.is_empty() {
        let mut hierarchy = CLASS_HIERARCHY.lock().unwrap();
        hierarchy.insert(class_name.to_string(), parent_class_name.to_string());
    }
    
    debug!("Created class {} with ID {} (is_actor: {})", class_name, class_id, is_actor);
    true
}

/// Generate a class ID from a class name
fn generate_class_id(class_name: &str) -> u32 {
    // Simple hash function for demonstration
    let mut hash: u32 = 5381;
    for c in class_name.bytes() {
        hash = ((hash << 5).wrapping_add(hash)).wrapping_add(c as u32);
    }
    hash
}

/// Get the class ID for a given class name
pub fn get_class_id_by_name(class_name: &str) -> Option<u32> {
    let class_map = CLASS_ID_MAP.lock().unwrap();
    class_map.get(class_name).copied()
}

/// Check if a class ID represents an actor class
pub fn is_actor_class(class_id: u32) -> bool {
    let actor_map = ACTOR_CLASS_MAP.lock().unwrap();
    actor_map.get(&class_id).copied().unwrap_or(false)
}

/// Register a class with the system
pub fn register_class(class_name: &str, class_id: u32, is_actor: bool) {
    {
        let mut class_map = CLASS_ID_MAP.lock().unwrap();
        class_map.insert(class_name.to_string(), class_id);
    }
    {
        let mut actor_map = ACTOR_CLASS_MAP.lock().unwrap();
        actor_map.insert(class_id, is_actor);
    }
}

/// Get the class name for a given class ID
pub fn get_class_name_by_id(class_id: u32) -> Option<String> {
    let class_map = CLASS_ID_MAP.lock().unwrap();
    for (name, id) in class_map.iter() {
        if *id == class_id {
            return Some(name.clone());
        }
    }
    None
}

/// Get the parent class name for a given class name
pub fn get_parent_class_name(class_name: &str) -> Option<String> {
    let hierarchy = CLASS_HIERARCHY.lock().unwrap();
    hierarchy.get(class_name).cloned()
}

/// Check if a class is derived from another class
pub fn is_class_derived_from(class_name: &str, potential_parent: &str) -> bool {
    if class_name == potential_parent {
        return true;
    }
    
    let mut current_class = class_name.to_string();
    
    // Follow the inheritance chain upwards
    while let Some(parent) = get_parent_class_name(&current_class) {
        if parent == potential_parent {
            return true;
        }
        current_class = parent;
    }
    
    false
} 