//! # Network Relevancy System
//!
//! Handles determining which actors and objects are relevant to which clients.
//! This allows the server to only send updates for objects that matter to a client,
//! reducing bandwidth usage and improving performance.
//!
//! The system supports multiple relevancy strategies:
//! - Always relevant (objects that all clients need to know about)
//! - Owner-only relevancy (only relevant to the owner)
//! - Distance-based relevancy (only relevant to clients within a certain distance)
//! - Zone-based relevancy (only relevant to clients in the same "zone")
//! - Custom relevancy logic

use spacetimedb::{ReducerContext, Address, Identity, SpacetimeTable, TableType};
use std::collections::{HashMap, HashSet};

use crate::object::ObjectId;
use crate::property::{PropertyChangeTracker, PropertyReplicator};
use crate::connection::ClientConnection;

// Re-export submodules
pub mod zone;
pub mod distance;
pub mod integration;

// Import shared types from the SharedModule
use spacetime_shared::relevancy::{
    RelevancyLevel,
    RelevancySettings,
    UpdateFrequency,
    NetworkPriority,
    ZoneMembership,
};

/// Flag to control the visibility of an object
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisibilityFlag {
    /// Object is visible to clients
    Visible,
    /// Object is hidden from clients
    Hidden,
    /// Object's visibility is determined by relevancy settings
    UseRelevancy,
}

/// Table to store relevancy zones
#[derive(TableType)]
pub struct RelevancyZone {
    /// Unique zone identifier
    pub zone_id: u32,
    
    /// Zone name for debugging
    pub name: String,
    
    /// Whether zone is active
    pub active: bool,
    
    /// Optional owner client address
    pub owner_address: Option<Address>,
}

/// Table to store zone memberships
#[derive(TableType)]
pub struct ZoneMembershipEntry {
    /// Entity ID (client ID or object ID)
    pub entity_id: u64,
    
    /// Is this entity a client
    pub is_client: bool,
    
    /// Zone ID this entity belongs to
    pub zone_id: u32,
}

/// Table to store relevancy settings
#[derive(TableType)]
pub struct RelevancySettingsEntry {
    /// Object these settings apply to
    pub object_id: ObjectId,
    
    /// Relevancy level as string
    pub relevancy_level: String,
    
    /// Update frequency as string
    pub update_frequency: String,
    
    /// Network priority as string
    pub network_priority: String,
    
    /// Maximum relevancy distance (if distance-based)
    pub max_distance: Option<f32>,
}

/// Converts string representations back to enum values
impl RelevancySettingsEntry {
    pub fn get_relevancy_level(&self) -> RelevancyLevel {
        match self.relevancy_level.as_str() {
            "AlwaysRelevant" => RelevancyLevel::AlwaysRelevant,
            "OwnerOnly" => RelevancyLevel::OwnerOnly,
            "DistanceBased" => RelevancyLevel::DistanceBased,
            "SameZone" => RelevancyLevel::SameZone,
            "Custom" => RelevancyLevel::Custom,
            "NeverRelevant" => RelevancyLevel::NeverRelevant,
            _ => RelevancyLevel::AlwaysRelevant, // Default
        }
    }
    
    pub fn get_update_frequency(&self) -> UpdateFrequency {
        match self.update_frequency.as_str() {
            "High" => UpdateFrequency::High,
            "Medium" => UpdateFrequency::Medium,
            "Low" => UpdateFrequency::Low,
            "OnDemand" => UpdateFrequency::OnDemand,
            _ => UpdateFrequency::Medium, // Default
        }
    }
    
    pub fn get_network_priority(&self) -> NetworkPriority {
        match self.network_priority.as_str() {
            "Critical" => NetworkPriority::Critical,
            "High" => NetworkPriority::High,
            "Normal" => NetworkPriority::Normal,
            "Low" => NetworkPriority::Low,
            _ => NetworkPriority::Normal, // Default
        }
    }
}

/// Core relevancy manager that determines which objects should be replicated to which clients
pub struct RelevancyManager {
    /// Cached relevancy settings for quick access
    relevancy_settings: HashMap<ObjectId, RelevancySettings>,
    
    /// Cached zone memberships for quick access
    zone_memberships: HashMap<u64, HashSet<u32>>,
    
    /// Cache of which objects are relevant to which clients
    relevancy_cache: HashMap<u64, HashSet<ObjectId>>,
    
    /// Tick counter for update frequency
    tick_counter: u32,
}

impl RelevancyManager {
    /// Create a new relevancy manager
    pub fn new() -> Self {
        Self {
            relevancy_settings: HashMap::new(),
            zone_memberships: HashMap::new(),
            relevancy_cache: HashMap::new(),
            tick_counter: 0,
        }
    }
    
    /// Initialize the relevancy system
    pub fn init(ctx: &ReducerContext) {
        // Create initial zones if needed
        if RelevancyZone::iter(ctx).next().is_none() {
            // Create a global zone that everything starts in
            RelevancyZone::insert_new(
                ctx,
                RelevancyZone {
                    zone_id: 0,
                    name: "Global".to_string(),
                    active: true,
                    owner_address: None,
                },
            ).expect("Failed to create global zone");
        }
    }
    
    /// Update the relevancy cache based on current state
    pub fn update_relevancy_cache(&mut self, ctx: &ReducerContext) {
        // Increment tick counter
        self.tick_counter = self.tick_counter.wrapping_add(1);
        
        // Clear the relevancy cache
        self.relevancy_cache.clear();
        
        // Reload settings from database if needed
        self.reload_settings_if_needed(ctx);
        
        // Process each client
        for client in ClientConnection::iter(ctx) {
            let client_id = client.client_id;
            let mut relevant_objects = HashSet::new();
            
            // Get the client's zones
            let client_zones = self.get_entity_zones(client_id, true);
            
            // For each object with relevancy settings
            for (object_id, settings) in &self.relevancy_settings {
                // Check if this object should be replicated to this client
                if self.is_object_relevant_to_client(*object_id, client_id, client_zones.clone()) {
                    // Check if update frequency matches the current tick
                    if self.should_update_on_current_tick(&settings.frequency) {
                        relevant_objects.insert(*object_id);
                    }
                }
            }
            
            // Store the relevant objects for this client
            self.relevancy_cache.insert(client_id, relevant_objects);
        }
    }
    
    /// Check if an object should update on the current tick based on frequency
    fn should_update_on_current_tick(&self, frequency: &UpdateFrequency) -> bool {
        match frequency {
            UpdateFrequency::High => true,
            UpdateFrequency::Medium => self.tick_counter % 2 == 0,
            UpdateFrequency::Low => self.tick_counter % 4 == 0,
            UpdateFrequency::OnDemand => false, // Only updates when explicitly requested
        }
    }
    
    /// Check if an object is currently relevant to a client
    fn is_object_relevant_to_client(
        &self,
        object_id: ObjectId,
        client_id: u64, 
        client_zones: HashSet<u32>
    ) -> bool {
        // Get the object's relevancy settings
        if let Some(settings) = self.relevancy_settings.get(&object_id) {
            match settings.level {
                // Always relevant to all clients
                RelevancyLevel::AlwaysRelevant => true,
                
                // Only relevant to the owner client
                RelevancyLevel::OwnerOnly => {
                    // TODO: Check if client is owner of this object
                    // For now, assume not owner
                    false
                },
                
                // Distance-based relevancy
                RelevancyLevel::DistanceBased => {
                    // Handle in distance.rs module
                    distance::is_within_relevant_distance(object_id, client_id, settings.max_distance)
                },
                
                // Zone-based relevancy
                RelevancyLevel::SameZone => {
                    // Get the object's zones
                    let object_zones = self.get_entity_zones(object_id, false);
                    
                    // Check if any of the client's zones match any of the object's zones
                    !client_zones.is_disjoint(&object_zones)
                },
                
                // Custom relevancy logic
                RelevancyLevel::Custom => {
                    // Handled by custom logic implemented elsewhere
                    // For now, default to relevant
                    true
                },
                
                // Never relevant to any client
                RelevancyLevel::NeverRelevant => false,
            }
        } else {
            // If no settings are defined, default to always relevant
            true
        }
    }
    
    /// Get the zones an entity (client or object) belongs to
    fn get_entity_zones(&self, entity_id: u64, is_client: bool) -> HashSet<u32> {
        // Check the cache first
        if let Some(zones) = self.zone_memberships.get(&entity_id) {
            return zones.clone();
        }
        
        // Empty set for now
        HashSet::new()
    }
    
    /// Reload settings from the database if needed
    fn reload_settings_if_needed(&mut self, ctx: &ReducerContext) {
        // For now, always reload (could be optimized to only reload when changes detected)
        self.reload_settings(ctx);
    }
    
    /// Reload all settings from the database
    fn reload_settings(&mut self, ctx: &ReducerContext) {
        // Clear existing settings
        self.relevancy_settings.clear();
        self.zone_memberships.clear();
        
        // Load relevancy settings
        for entry in RelevancySettingsEntry::iter(ctx) {
            let settings = RelevancySettings {
                object_id: entry.object_id,
                level: entry.get_relevancy_level(),
                frequency: entry.get_update_frequency(),
                priority: entry.get_network_priority(),
                max_distance: entry.max_distance,
            };
            
            self.relevancy_settings.insert(entry.object_id, settings);
        }
        
        // Load zone memberships
        for entry in ZoneMembershipEntry::iter(ctx) {
            let entity_id = entry.entity_id;
            let zone_id = entry.zone_id;
            
            self.zone_memberships
                .entry(entity_id)
                .or_insert_with(HashSet::new)
                .insert(zone_id);
        }
    }
    
    /// Get objects relevant to a specific client
    pub fn get_relevant_objects_for_client(&self, client_id: u64) -> Option<&HashSet<ObjectId>> {
        self.relevancy_cache.get(&client_id)
    }
    
    /// Add an entity to a zone
    pub fn add_to_zone(ctx: &ReducerContext, entity_id: u64, is_client: bool, zone_id: u32) -> Result<(), String> {
        // Check if the zone exists
        if RelevancyZone::filter_by_zone_id(ctx, zone_id).next().is_none() {
            return Err(format!("Zone with ID {} does not exist", zone_id));
        }
        
        // Check if the entity is already in this zone
        let already_in_zone = ZoneMembershipEntry::iter(ctx)
            .any(|entry| entry.entity_id == entity_id && entry.zone_id == zone_id);
            
        if !already_in_zone {
            // Add the entity to the zone
            ZoneMembershipEntry::insert_new(
                ctx,
                ZoneMembershipEntry {
                    entity_id,
                    is_client,
                    zone_id,
                },
            ).map_err(|e| format!("Failed to add entity to zone: {}", e))?;
        }
        
        Ok(())
    }
    
    /// Remove an entity from a zone
    pub fn remove_from_zone(ctx: &ReducerContext, entity_id: u64, zone_id: u32) -> Result<(), String> {
        // Find the entry to delete
        let entries_to_delete: Vec<_> = ZoneMembershipEntry::iter(ctx)
            .filter(|entry| entry.entity_id == entity_id && entry.zone_id == zone_id)
            .collect();
            
        // Delete each matching entry
        for entry in entries_to_delete {
            ZoneMembershipEntry::delete_by_entity_id_and_zone_id(ctx, entry.entity_id, entry.zone_id)
                .map_err(|e| format!("Failed to remove entity from zone: {}", e))?;
        }
        
        Ok(())
    }
    
    /// Update or set relevancy settings for an object
    pub fn set_relevancy_settings(
        ctx: &ReducerContext,
        object_id: ObjectId,
        level: RelevancyLevel,
        frequency: UpdateFrequency,
        priority: NetworkPriority,
        max_distance: Option<f32>,
    ) -> Result<(), String> {
        // Convert enum values to strings
        let relevancy_level = match level {
            RelevancyLevel::AlwaysRelevant => "AlwaysRelevant",
            RelevancyLevel::OwnerOnly => "OwnerOnly",
            RelevancyLevel::DistanceBased => "DistanceBased",
            RelevancyLevel::SameZone => "SameZone",
            RelevancyLevel::Custom => "Custom",
            RelevancyLevel::NeverRelevant => "NeverRelevant",
        };
        
        let update_frequency = match frequency {
            UpdateFrequency::High => "High",
            UpdateFrequency::Medium => "Medium",
            UpdateFrequency::Low => "Low",
            UpdateFrequency::OnDemand => "OnDemand",
        };
        
        let network_priority = match priority {
            NetworkPriority::Critical => "Critical",
            NetworkPriority::High => "High",
            NetworkPriority::Normal => "Normal",
            NetworkPriority::Low => "Low",
        };
        
        // Delete any existing settings for this object
        let existing = RelevancySettingsEntry::filter_by_object_id(ctx, object_id);
        for entry in existing {
            RelevancySettingsEntry::delete_by_object_id(ctx, entry.object_id)
                .map_err(|e| format!("Failed to delete existing relevancy settings: {}", e))?;
        }
        
        // Insert new settings
        RelevancySettingsEntry::insert_new(
            ctx,
            RelevancySettingsEntry {
                object_id,
                relevancy_level: relevancy_level.to_string(),
                update_frequency: update_frequency.to_string(),
                network_priority: network_priority.to_string(),
                max_distance,
            },
        ).map_err(|e| format!("Failed to set relevancy settings: {}", e))?;
        
        Ok(())
    }
}

// Static instance for global access
static mut RELEVANCY_MANAGER: Option<RelevancyManager> = None;

/// Get the global relevancy manager instance
pub fn get_relevancy_manager() -> &'static mut RelevancyManager {
    unsafe {
        if RELEVANCY_MANAGER.is_none() {
            RELEVANCY_MANAGER = Some(RelevancyManager::new());
        }
        RELEVANCY_MANAGER.as_mut().unwrap()
    }
}

/// Initialize the relevancy system
pub fn init(ctx: &ReducerContext) {
    // Initialize the relevancy manager
    let manager = get_relevancy_manager();
    manager.init(ctx);
}

/// Update relevancy state (called periodically)
pub fn update(ctx: &ReducerContext) {
    // Update the relevancy cache
    let manager = get_relevancy_manager();
    manager.update_relevancy_cache(ctx);
}

/// Filter a list of objects to only include those relevant to a client
pub fn filter_relevant_objects(
    client_id: u64,
    object_ids: &[ObjectId],
) -> Vec<ObjectId> {
    let manager = get_relevancy_manager();
    
    if let Some(relevant_objects) = manager.get_relevant_objects_for_client(client_id) {
        object_ids
            .iter()
            .filter(|id| relevant_objects.contains(id))
            .cloned()
            .collect()
    } else {
        // If client isn't in the cache, assume all objects are relevant
        object_ids.to_vec()
    }
}

/// Check if a specific object is relevant to a client
pub fn is_object_relevant_to_client(object_id: ObjectId, client_id: u64) -> bool {
    let manager = get_relevancy_manager();
    
    if let Some(relevant_objects) = manager.get_relevant_objects_for_client(client_id) {
        relevant_objects.contains(&object_id)
    } else {
        // If client isn't in the cache, default to true
        true
    }
} 