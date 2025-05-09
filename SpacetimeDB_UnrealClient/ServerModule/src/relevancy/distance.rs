//! # Distance-Based Relevancy
//!
//! This module handles distance-based relevancy calculations, determining if
//! objects are relevant to clients based on their spatial distance.

use spacetimedb::ReducerContext;
use std::collections::HashMap;

use crate::object::ObjectId;
use crate::property::{PropertyType, PropertyValue};

/// Simple 3D vector for position calculations
#[derive(Debug, Clone, Copy)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    /// Create a new vector from components
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    
    /// Calculate squared distance between two points
    pub fn distance_squared(&self, other: &Vector3) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        dx * dx + dy * dy + dz * dz
    }
    
    /// Calculate distance between two points
    pub fn distance(&self, other: &Vector3) -> f32 {
        self.distance_squared(other).sqrt()
    }
}

/// Cache of entity positions for efficient distance calculations
pub struct SpatialCache {
    /// Maps entity IDs to their current positions
    positions: HashMap<u64, Vector3>,
}

impl SpatialCache {
    /// Create a new empty spatial cache
    pub fn new() -> Self {
        Self {
            positions: HashMap::new(),
        }
    }
    
    /// Update the position of an entity in the cache
    pub fn update_position(&mut self, entity_id: u64, position: Vector3) {
        self.positions.insert(entity_id, position);
    }
    
    /// Get the position of an entity from the cache
    pub fn get_position(&self, entity_id: u64) -> Option<&Vector3> {
        self.positions.get(&entity_id)
    }
    
    /// Calculate distance between two entities
    pub fn distance_between(&self, entity1_id: u64, entity2_id: u64) -> Option<f32> {
        match (self.get_position(entity1_id), self.get_position(entity2_id)) {
            (Some(pos1), Some(pos2)) => Some(pos1.distance(pos2)),
            _ => None,
        }
    }
    
    /// Check if two entities are within a specified distance
    pub fn are_within_distance(&self, entity1_id: u64, entity2_id: u64, max_distance: f32) -> bool {
        match (self.get_position(entity1_id), self.get_position(entity2_id)) {
            (Some(pos1), Some(pos2)) => {
                // Use squared distance for efficiency (avoid square root)
                let max_distance_squared = max_distance * max_distance;
                pos1.distance_squared(pos2) <= max_distance_squared
            },
            _ => false,
        }
    }
    
    /// Remove an entity from the cache
    pub fn remove_entity(&mut self, entity_id: u64) {
        self.positions.remove(&entity_id);
    }
    
    /// Clear the entire cache
    pub fn clear(&mut self) {
        self.positions.clear();
    }
}

// Static instance for global access
static mut SPATIAL_CACHE: Option<SpatialCache> = None;

/// Get the global spatial cache instance
pub fn get_spatial_cache() -> &'static mut SpatialCache {
    unsafe {
        if SPATIAL_CACHE.is_none() {
            SPATIAL_CACHE = Some(SpatialCache::new());
        }
        SPATIAL_CACHE.as_mut().unwrap()
    }
}

/// Extract a Vector3 from a property value if it's a position
fn extract_position(property_value: &PropertyValue) -> Option<Vector3> {
    match property_value {
        PropertyValue::Vector { x, y, z } => {
            Some(Vector3::new(*x, *y, *z))
        },
        PropertyValue::Transform { 
            pos_x, pos_y, pos_z, 
            rot_x: _, rot_y: _, rot_z: _, rot_w: _,
            scale_x: _, scale_y: _, scale_z: _,
        } => {
            Some(Vector3::new(*pos_x, *pos_y, *pos_z))
        },
        _ => None,
    }
}

/// Update the position of an entity in the spatial cache based on a property update
pub fn update_entity_position(
    entity_id: u64,
    property_name: &str,
    property_value: &PropertyValue,
) {
    // Only update if this is a position property
    if property_name == "Location" || property_name == "Position" || property_name == "Transform" {
        if let Some(position) = extract_position(property_value) {
            let cache = get_spatial_cache();
            cache.update_position(entity_id, position);
        }
    }
}

/// Check if an object is within relevant distance of a client
pub fn is_within_relevant_distance(
    object_id: ObjectId,
    client_id: u64,
    max_distance: Option<f32>,
) -> bool {
    let cache = get_spatial_cache();
    
    // If no max distance is specified, default to a large value
    let max_distance = max_distance.unwrap_or(10000.0);
    
    // Check if the object and client are within the specified distance
    cache.are_within_distance(object_id, client_id, max_distance)
}

/// Initialize the distance-based relevancy system
pub fn init(_ctx: &ReducerContext) {
    // Initialize the spatial cache
    let _cache = get_spatial_cache();
    // Nothing else to initialize for now
}

/// Update the distance-based relevancy system
pub fn update(_ctx: &ReducerContext) {
    // Nothing to do here yet, positions are updated as properties change
}

/// Clear inactive entities from the spatial cache
pub fn cleanup_inactive_entities(inactive_entity_ids: &[u64]) {
    let cache = get_spatial_cache();
    
    for &entity_id in inactive_entity_ids {
        cache.remove_entity(entity_id);
    }
}

/// Get all entities within a certain distance of a point
pub fn get_entities_within_distance(
    position: Vector3,
    max_distance: f32,
) -> Vec<u64> {
    let cache = get_spatial_cache();
    let max_distance_squared = max_distance * max_distance;
    
    let mut result = Vec::new();
    
    for (&entity_id, &entity_pos) in &cache.positions {
        if position.distance_squared(&entity_pos) <= max_distance_squared {
            result.push(entity_id);
        }
    }
    
    result
}

/// Get all client IDs that are within a certain distance of an entity
pub fn get_clients_within_distance(
    entity_id: u64,
    max_distance: f32,
    client_ids: &[u64],
) -> Vec<u64> {
    let cache = get_spatial_cache();
    
    if let Some(entity_pos) = cache.get_position(entity_id) {
        client_ids
            .iter()
            .filter(|&&client_id| {
                if let Some(client_pos) = cache.get_position(client_id) {
                    entity_pos.distance(client_pos) <= max_distance
                } else {
                    false
                }
            })
            .copied()
            .collect()
    } else {
        Vec::new()
    }
} 