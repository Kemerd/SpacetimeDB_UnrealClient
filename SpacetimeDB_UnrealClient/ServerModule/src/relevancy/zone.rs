//! # Zone-Based Relevancy
//!
//! This module handles zone-based relevancy, allowing objects and clients
//! to be organized into zones. Objects in a zone are relevant to clients in the same zone.

use spacetimedb::{ReducerContext, TableType, StageReducer, Address, SpacetimeTable};
use std::collections::{HashMap, HashSet};

use crate::object::ObjectId;
use crate::relevancy::{RelevancyZone, ZoneMembershipEntry};

/// Create a new relevancy zone
#[reducer]
pub fn create_zone(
    ctx: StageReducer, 
    name: String,
    active: bool,
) -> Result<u32, String> {
    // Generate a new zone ID
    let zone_id = generate_zone_id(ctx.ctx());
    
    // Create the zone
    let zone = RelevancyZone {
        zone_id,
        name,
        active,
        owner_address: Some(ctx.sender()),
    };
    
    // Insert the zone
    RelevancyZone::insert_new(ctx.ctx(), zone).map_err(|e| format!("Failed to create zone: {}", e))?;
    
    // Return the new zone ID
    Ok(zone_id)
}

/// Update a relevancy zone
#[reducer]
pub fn update_zone(
    ctx: StageReducer,
    zone_id: u32,
    name: Option<String>,
    active: Option<bool>,
) -> Result<(), String> {
    // Get the zone
    let zone = match RelevancyZone::filter_by_zone_id(ctx.ctx(), zone_id).next() {
        Some(zone) => zone,
        None => return Err(format!("Zone with ID {} does not exist", zone_id)),
    };
    
    // Check if the sender is the owner or an admin
    if let Some(owner) = zone.owner_address {
        if owner != ctx.sender() && !is_admin(ctx.ctx(), ctx.sender()) {
            return Err("You don't have permission to update this zone".to_string());
        }
    }
    
    // Update the zone
    let updated_zone = RelevancyZone {
        zone_id,
        name: name.unwrap_or(zone.name),
        active: active.unwrap_or(zone.active),
        owner_address: zone.owner_address,
    };
    
    // Delete the old zone
    RelevancyZone::delete_by_zone_id(ctx.ctx(), zone_id).map_err(|e| format!("Failed to update zone: {}", e))?;
    
    // Insert the updated zone
    RelevancyZone::insert_new(ctx.ctx(), updated_zone).map_err(|e| format!("Failed to update zone: {}", e))?;
    
    Ok(())
}

/// Delete a relevancy zone
#[reducer]
pub fn delete_zone(
    ctx: StageReducer,
    zone_id: u32,
) -> Result<(), String> {
    // Can't delete the global zone (zone_id 0)
    if zone_id == 0 {
        return Err("Cannot delete the global zone".to_string());
    }
    
    // Get the zone
    let zone = match RelevancyZone::filter_by_zone_id(ctx.ctx(), zone_id).next() {
        Some(zone) => zone,
        None => return Err(format!("Zone with ID {} does not exist", zone_id)),
    };
    
    // Check if the sender is the owner or an admin
    if let Some(owner) = zone.owner_address {
        if owner != ctx.sender() && !is_admin(ctx.ctx(), ctx.sender()) {
            return Err("You don't have permission to delete this zone".to_string());
        }
    }
    
    // Delete the zone
    RelevancyZone::delete_by_zone_id(ctx.ctx(), zone_id).map_err(|e| format!("Failed to delete zone: {}", e))?;
    
    // Remove all entities from this zone
    let members: Vec<_> = ZoneMembershipEntry::iter(ctx.ctx())
        .filter(|entry| entry.zone_id == zone_id)
        .map(|entry| (entry.entity_id, entry.zone_id))
        .collect();
        
    for (entity_id, zone_id) in members {
        ZoneMembershipEntry::delete_by_entity_id_and_zone_id(ctx.ctx(), entity_id, zone_id)
            .map_err(|e| format!("Failed to clean up zone membership: {}", e))?;
    }
    
    Ok(())
}

/// Add an entity to a zone
#[reducer]
pub fn add_entity_to_zone(
    ctx: StageReducer,
    entity_id: u64,
    is_client: bool,
    zone_id: u32,
) -> Result<(), String> {
    // Check if the zone exists
    if RelevancyZone::filter_by_zone_id(ctx.ctx(), zone_id).next().is_none() {
        return Err(format!("Zone with ID {} does not exist", zone_id));
    }
    
    // Check if the entity is already in this zone
    let already_in_zone = ZoneMembershipEntry::iter(ctx.ctx())
        .any(|entry| entry.entity_id == entity_id && entry.zone_id == zone_id);
        
    if already_in_zone {
        return Ok(()); // Already in zone, nothing to do
    }
    
    // Add the entity to the zone
    ZoneMembershipEntry::insert_new(
        ctx.ctx(),
        ZoneMembershipEntry {
            entity_id,
            is_client,
            zone_id,
        },
    ).map_err(|e| format!("Failed to add entity to zone: {}", e))?;
    
    Ok(())
}

/// Remove an entity from a zone
#[reducer]
pub fn remove_entity_from_zone(
    ctx: StageReducer,
    entity_id: u64,
    zone_id: u32,
) -> Result<(), String> {
    // Find the entry to delete
    let entries_to_delete: Vec<_> = ZoneMembershipEntry::iter(ctx.ctx())
        .filter(|entry| entry.entity_id == entity_id && entry.zone_id == zone_id)
        .map(|entry| (entry.entity_id, entry.zone_id))
        .collect();
        
    if entries_to_delete.is_empty() {
        return Ok(()); // Entity not in this zone, nothing to do
    }
    
    // Delete each matching entry
    for (entity_id, zone_id) in entries_to_delete {
        ZoneMembershipEntry::delete_by_entity_id_and_zone_id(ctx.ctx(), entity_id, zone_id)
            .map_err(|e| format!("Failed to remove entity from zone: {}", e))?;
    }
    
    Ok(())
}

/// Add multiple entities to a zone
#[reducer]
pub fn add_entities_to_zone(
    ctx: StageReducer,
    entity_ids: Vec<u64>,
    are_clients: Vec<bool>,
    zone_id: u32,
) -> Result<(), String> {
    // Check if the inputs are valid
    if entity_ids.len() != are_clients.len() {
        return Err("Entity IDs and client flags must have the same length".to_string());
    }
    
    // Check if the zone exists
    if RelevancyZone::filter_by_zone_id(ctx.ctx(), zone_id).next().is_none() {
        return Err(format!("Zone with ID {} does not exist", zone_id));
    }
    
    // Add each entity to the zone
    for i in 0..entity_ids.len() {
        let entity_id = entity_ids[i];
        let is_client = are_clients[i];
        
        // Check if the entity is already in this zone
        let already_in_zone = ZoneMembershipEntry::iter(ctx.ctx())
            .any(|entry| entry.entity_id == entity_id && entry.zone_id == zone_id);
            
        if !already_in_zone {
            // Add the entity to the zone
            ZoneMembershipEntry::insert_new(
                ctx.ctx(),
                ZoneMembershipEntry {
                    entity_id,
                    is_client,
                    zone_id,
                },
            ).map_err(|e| format!("Failed to add entity {} to zone: {}", entity_id, e))?;
        }
    }
    
    Ok(())
}

/// Get the zones an entity belongs to
#[reducer]
pub fn get_entity_zones(
    ctx: StageReducer,
    entity_id: u64,
) -> Vec<u32> {
    ZoneMembershipEntry::iter(ctx.ctx())
        .filter(|entry| entry.entity_id == entity_id)
        .map(|entry| entry.zone_id)
        .collect()
}

/// Get all entities in a zone
#[reducer]
pub fn get_zone_entities(
    ctx: StageReducer,
    zone_id: u32,
) -> Vec<u64> {
    ZoneMembershipEntry::iter(ctx.ctx())
        .filter(|entry| entry.zone_id == zone_id)
        .map(|entry| entry.entity_id)
        .collect()
}

/// Generate a new unique zone ID
fn generate_zone_id(ctx: &ReducerContext) -> u32 {
    // Find the highest existing zone ID
    let max_id = RelevancyZone::iter(ctx)
        .map(|zone| zone.zone_id)
        .max()
        .unwrap_or(0);
    
    // Return one higher
    max_id + 1
}

/// Check if an address belongs to an admin
fn is_admin(_ctx: &ReducerContext, _address: Address) -> bool {
    // In a real implementation, this would check against an admin list
    // For now, return false (no admins)
    false
}

/// Get all entities that share a zone with a specific entity
pub fn get_entities_in_same_zones(
    ctx: &ReducerContext,
    entity_id: u64,
) -> HashSet<u64> {
    let mut result = HashSet::new();
    
    // Get the zones this entity belongs to
    let entity_zones: HashSet<u32> = ZoneMembershipEntry::iter(ctx)
        .filter(|entry| entry.entity_id == entity_id)
        .map(|entry| entry.zone_id)
        .collect();
    
    // If the entity isn't in any zones, return empty set
    if entity_zones.is_empty() {
        return result;
    }
    
    // Get all entities in those zones
    for entry in ZoneMembershipEntry::iter(ctx) {
        if entity_zones.contains(&entry.zone_id) && entry.entity_id != entity_id {
            result.insert(entry.entity_id);
        }
    }
    
    result
}

/// Check if two entities share any zones
pub fn share_any_zone(
    ctx: &ReducerContext,
    entity1_id: u64,
    entity2_id: u64,
) -> bool {
    // Get the zones each entity belongs to
    let entity1_zones: HashSet<u32> = ZoneMembershipEntry::iter(ctx)
        .filter(|entry| entry.entity_id == entity1_id)
        .map(|entry| entry.zone_id)
        .collect();
    
    let entity2_zones: HashSet<u32> = ZoneMembershipEntry::iter(ctx)
        .filter(|entry| entry.entity_id == entity2_id)
        .map(|entry| entry.zone_id)
        .collect();
    
    // Check if there's any overlap
    !entity1_zones.is_disjoint(&entity2_zones)
}

/// Get all active zones
pub fn get_active_zones(ctx: &ReducerContext) -> Vec<RelevancyZone> {
    RelevancyZone::iter(ctx)
        .filter(|zone| zone.active)
        .collect()
}

/// Add a client to default zones when they connect
pub fn add_client_to_default_zones(
    ctx: &ReducerContext,
    client_id: u64,
) -> Result<(), String> {
    // Add the client to the global zone (zone_id 0)
    ZoneMembershipEntry::insert_new(
        ctx,
        ZoneMembershipEntry {
            entity_id: client_id,
            is_client: true,
            zone_id: 0,
        },
    ).map_err(|e| format!("Failed to add client to global zone: {}", e))?;
    
    Ok(())
}

/// Remove a client from all zones when they disconnect
pub fn remove_client_from_all_zones(
    ctx: &ReducerContext,
    client_id: u64,
) -> Result<(), String> {
    // Find all zone memberships for this client
    let memberships: Vec<_> = ZoneMembershipEntry::iter(ctx)
        .filter(|entry| entry.entity_id == client_id && entry.is_client)
        .map(|entry| entry.zone_id)
        .collect();
    
    // Remove the client from each zone
    for zone_id in memberships {
        ZoneMembershipEntry::delete_by_entity_id_and_zone_id(ctx, client_id, zone_id)
            .map_err(|e| format!("Failed to remove client from zone {}: {}", zone_id, e))?;
    }
    
    Ok(())
} 