//! # Class System
//! 
//! This module provides functionality for working with classes in the system.
//! It handles class IDs, actor class detection, and other class-related operations.

use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;

/// Map of class names to class IDs
static CLASS_ID_MAP: Lazy<Mutex<HashMap<String, u32>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Map of class IDs to information about whether it's an actor class
static ACTOR_CLASS_MAP: Lazy<Mutex<HashMap<u32, bool>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

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