//! # Property Replication
//!
//! Handles the replication of properties between server and clients, managing
//! property change detection, prioritization, and network synchronization.

use crate::actor::Actor;
use crate::client::ClientInfo;
use crate::object::{ObjectId, UObject};
use crate::property::{PropertyType, PropertyValue};
use spacetimedb_sdk::reducer::StageReducer;
use std::collections::{HashMap, HashSet};

/// Flags for property replication settings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// Represents a property that should be replicated
#[derive(Debug, Clone)]
pub struct ReplicatedProperty {
    /// The name of the property
    pub name: String,
    /// The property type
    pub property_type: PropertyType,
    /// The replication condition
    pub condition: ReplicationCondition,
    /// Replication priority (higher = more frequent updates)
    pub priority: u8,
}

/// A collection of properties for a class that should be replicated
#[derive(Debug, Clone)]
pub struct ReplicatedPropertySet {
    /// The name of the class
    pub class_name: String,
    /// The properties to replicate
    pub properties: Vec<ReplicatedProperty>,
}

/// A registry of properties that should be replicated for each class
#[derive(Debug, Default)]
pub struct ReplicationRegistry {
    /// Map of class name to its replicated properties
    class_properties: HashMap<String, ReplicatedPropertySet>,
}

impl ReplicationRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            class_properties: HashMap::new(),
        }
    }
    
    /// Register a set of replicated properties for a class
    pub fn register_class(&mut self, property_set: ReplicatedPropertySet) {
        self.class_properties.insert(property_set.class_name.clone(), property_set);
    }
    
    /// Get the replicated properties for a class
    pub fn get_class_properties(&self, class_name: &str) -> Option<&ReplicatedPropertySet> {
        self.class_properties.get(class_name)
    }
}

/// Tracks changes to properties that need to be replicated
#[derive(Debug, Default)]
pub struct PropertyChangeTracker {
    /// Map of object ID to changed properties
    changes: HashMap<ObjectId, HashMap<String, PropertyValue>>,
    /// Set of object IDs that have been created since the last replication
    new_objects: HashSet<ObjectId>,
    /// Set of object IDs that have been destroyed since the last replication
    destroyed_objects: HashSet<ObjectId>,
}

impl PropertyChangeTracker {
    /// Create a new empty tracker
    pub fn new() -> Self {
        Self {
            changes: HashMap::new(),
            new_objects: HashSet::new(),
            destroyed_objects: HashSet::new(),
        }
    }
    
    /// Record a property change
    pub fn record_change(&mut self, object_id: ObjectId, property_name: &str, value: PropertyValue) {
        self.changes
            .entry(object_id)
            .or_insert_with(HashMap::new)
            .insert(property_name.to_string(), value);
    }
    
    /// Record a new object
    pub fn record_new_object(&mut self, object_id: ObjectId) {
        self.new_objects.insert(object_id);
    }
    
    /// Record a destroyed object
    pub fn record_destroyed_object(&mut self, object_id: ObjectId) {
        self.destroyed_objects.insert(object_id);
        // Remove any pending changes for this object
        self.changes.remove(&object_id);
    }
    
    /// Get all changes for an object
    pub fn get_object_changes(&self, object_id: ObjectId) -> Option<&HashMap<String, PropertyValue>> {
        self.changes.get(&object_id)
    }
    
    /// Get all new objects
    pub fn get_new_objects(&self) -> &HashSet<ObjectId> {
        &self.new_objects
    }
    
    /// Get all destroyed objects
    pub fn get_destroyed_objects(&self) -> &HashSet<ObjectId> {
        &self.destroyed_objects
    }
    
    /// Clear all tracked changes
    pub fn clear(&mut self) {
        self.changes.clear();
        self.new_objects.clear();
        self.destroyed_objects.clear();
    }
}

/// Defines a snapshot of an object's state for replication
#[derive(Debug, Clone)]
pub struct ObjectStateSnapshot {
    /// The object ID
    pub object_id: ObjectId,
    /// The class name
    pub class_name: String,
    /// The object's properties
    pub properties: HashMap<String, PropertyValue>,
    /// Whether this is a new object (initial replication)
    pub is_new: bool,
}

/// Manages the replication of properties to clients
pub struct PropertyReplicator {
    /// Registry of properties to replicate
    registry: ReplicationRegistry,
    /// Tracks changes since last replication
    tracker: PropertyChangeTracker,
}

impl PropertyReplicator {
    /// Create a new property replicator
    pub fn new(registry: ReplicationRegistry) -> Self {
        Self {
            registry,
            tracker: PropertyChangeTracker::new(),
        }
    }
    
    /// Record a property change
    pub fn record_change(&mut self, object_id: ObjectId, property_name: &str, value: PropertyValue) {
        self.tracker.record_change(object_id, property_name, value);
    }
    
    /// Record a new object
    pub fn record_new_object(&mut self, object_id: ObjectId) {
        self.tracker.record_new_object(object_id);
    }
    
    /// Record a destroyed object
    pub fn record_destroyed_object(&mut self, object_id: ObjectId) {
        self.tracker.record_destroyed_object(object_id);
    }
    
    /// Generate snapshots of objects that need to be replicated to a client
    pub fn generate_snapshots_for_client(
        &self,
        client_id: u64,
        objects: &HashMap<ObjectId, UObject>,
    ) -> Vec<ObjectStateSnapshot> {
        let mut snapshots = Vec::new();
        
        // Add snapshots for new objects
        for &object_id in self.tracker.get_new_objects() {
            if let Some(object) = objects.get(&object_id) {
                // Skip if this is an owner-only object and client is not the owner
                if object.owner_client_id != 0 && object.owner_client_id != client_id {
                    continue;
                }
                
                let mut properties = HashMap::new();
                
                // Find properties to replicate for this class
                if let Some(prop_set) = self.registry.get_class_properties(&object.class_name) {
                    for prop in &prop_set.properties {
                        // Skip server-only properties
                        if prop.condition == ReplicationCondition::ServerOnly {
                            continue;
                        }
                        
                        // Skip owner-only properties if client is not the owner
                        if prop.condition == ReplicationCondition::OwnerOnly && 
                           object.owner_client_id != client_id {
                            continue;
                        }
                        
                        // Get the property value
                        if let Some(value) = object.properties.get(&prop.name) {
                            properties.insert(prop.name.clone(), value.clone());
                        }
                    }
                }
                
                snapshots.push(ObjectStateSnapshot {
                    object_id,
                    class_name: object.class_name.clone(),
                    properties,
                    is_new: true,
                });
            }
        }
        
        // Add snapshots for changed objects
        for (&object_id, changes) in &self.tracker.changes {
            // Skip new objects (already handled above)
            if self.tracker.new_objects.contains(&object_id) {
                continue;
            }
            
            // Skip destroyed objects
            if self.tracker.destroyed_objects.contains(&object_id) {
                continue;
            }
            
            if let Some(object) = objects.get(&object_id) {
                // Skip if this is an owner-only object and client is not the owner
                if object.owner_client_id != 0 && object.owner_client_id != client_id {
                    continue;
                }
                
                let mut properties = HashMap::new();
                
                // Find properties to replicate for this class
                if let Some(prop_set) = self.registry.get_class_properties(&object.class_name) {
                    for prop in &prop_set.properties {
                        // Skip if not in the changes and not Always condition
                        if !changes.contains_key(&prop.name) && 
                           prop.condition != ReplicationCondition::Always {
                            continue;
                        }
                        
                        // Skip server-only properties
                        if prop.condition == ReplicationCondition::ServerOnly {
                            continue;
                        }
                        
                        // Skip owner-only properties if client is not the owner
                        if prop.condition == ReplicationCondition::OwnerOnly && 
                           object.owner_client_id != client_id {
                            continue;
                        }
                        
                        // Get the property value (from changes or object)
                        if let Some(value) = changes.get(&prop.name)
                            .or_else(|| object.properties.get(&prop.name)) 
                        {
                            properties.insert(prop.name.clone(), value.clone());
                        }
                    }
                }
                
                // Only create a snapshot if there are properties to replicate
                if !properties.is_empty() {
                    snapshots.push(ObjectStateSnapshot {
                        object_id,
                        class_name: object.class_name.clone(),
                        properties,
                        is_new: false,
                    });
                }
            }
        }
        
        snapshots
    }
    
    /// Clear all tracked changes
    pub fn clear_changes(&mut self) {
        self.tracker.clear();
    }
}

/// A reducer function to send property updates to a client
#[reducer]
pub fn send_property_updates(ctx: StageReducer, client_id: u64) {
    // This would call into the client to send property updates
    // Implementation would depend on the specific client integration
}

/// A reducer function to request property update from the server
#[reducer]
pub fn request_property_update(ctx: StageReducer, client_id: u64, object_id: ObjectId) {
    // This would trigger a property update for a specific object
} 