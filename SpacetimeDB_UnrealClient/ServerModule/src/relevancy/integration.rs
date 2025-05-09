//! # Relevancy Integration
//!
//! This module integrates the relevancy system with property replication,
//! ensuring only relevant property updates are sent to each client.

use spacetimedb::{ReducerContext, TableType, SpacetimeTable};
use std::collections::{HashMap, HashSet};

use crate::object::ObjectId;
use crate::property::{PropertyValue, PropertyReplicator};
use crate::connection::ClientConnection;
use crate::relevancy::{filter_relevant_objects, is_object_relevant_to_client};

/// Filter function used by the property replication system to determine
/// which clients should receive updates for a specific object
pub fn filter_clients_for_object_update(
    ctx: &ReducerContext,
    object_id: ObjectId,
) -> Vec<u64> {
    // Get all connected clients
    let clients: Vec<u64> = ClientConnection::iter(ctx)
        .map(|conn| conn.client_id)
        .collect();
    
    // Filter to only clients for which this object is relevant
    clients
        .into_iter()
        .filter(|&client_id| is_object_relevant_to_client(object_id, client_id))
        .collect()
}

/// Filter objects that should be initially replicated to a client when they connect
pub fn filter_initial_objects_for_client(
    ctx: &ReducerContext,
    client_id: u64,
    all_objects: &[ObjectId],
) -> Vec<ObjectId> {
    // Use the relevancy system to filter the objects
    filter_relevant_objects(client_id, all_objects)
}

/// Hook called when a property value changes, to update relevancy if needed
pub fn on_property_changed(
    ctx: &ReducerContext,
    object_id: ObjectId,
    property_name: &str,
    property_value: &PropertyValue,
) {
    // Check if this property affects relevancy
    if is_relevancy_affecting_property(property_name) {
        // If it's a position property, update the spatial cache
        if is_position_property(property_name) {
            // Update distance-based relevancy
            crate::relevancy::distance::update_entity_position(
                object_id,
                property_name,
                property_value,
            );
        }
        
        // For other relevancy-affecting properties, we might need to update zone memberships
        // or other relevancy data, but that's implementation-specific
        
        // After updating relevancy data, we need to refresh the relevancy cache
        // This ensures subsequent replication decisions use the updated relevancy info
        crate::relevancy::update(ctx);
    }
}

/// Check if a property affects relevancy determination
fn is_relevancy_affecting_property(property_name: &str) -> bool {
    // Position properties affect distance-based relevancy
    if is_position_property(property_name) {
        return true;
    }
    
    // Other properties that might affect relevancy:
    // - Team or faction (might affect zone membership)
    // - Visibility flags
    // - Stealth or detection values
    match property_name {
        "Team" | "Faction" | "VisibilityFlag" | "StealthLevel" => true,
        _ => false,
    }
}

/// Check if a property represents position data
fn is_position_property(property_name: &str) -> bool {
    match property_name {
        "Location" | "Position" | "Transform" => true,
        _ => false,
    }
}

/// Hook into property replication to filter by relevancy
pub fn hook_into_property_replication(replicator: &mut PropertyReplicator) {
    // Add our filter function to the replicator
    // Note: In a real implementation, this would require the PropertyReplicator
    // to have a method to set a filter function
    // replicator.set_client_filter(filter_clients_for_object_update);
    
    // For now, we'll just add a comment here indicating this is a placeholder
    // The actual implementation would depend on how the PropertyReplicator is structured
}

/// Table to track which objects have been replicated to which clients
#[derive(TableType)]
pub struct ClientObjectReplication {
    /// Client ID
    pub client_id: u64,
    
    /// Object ID
    pub object_id: ObjectId,
    
    /// Whether initial replication has completed
    pub initial_replication_complete: bool,
    
    /// Last replication timestamp
    pub last_replication_time: u64,
}

/// Reconcile client replication state when relevancy changes
pub fn reconcile_client_replication_state(
    ctx: &ReducerContext,
    client_id: u64,
) -> Result<(), String> {
    // Get objects currently marked as replicated to this client
    let currently_replicated: HashSet<ObjectId> = ClientObjectReplication::iter(ctx)
        .filter(|entry| entry.client_id == client_id)
        .map(|entry| entry.object_id)
        .collect();
    
    // Get objects that should be relevant to this client
    let relevant_objects: HashSet<ObjectId> = match crate::relevancy::get_relevancy_manager()
        .get_relevant_objects_for_client(client_id)
    {
        Some(objects) => objects.clone(),
        None => HashSet::new(),
    };
    
    // Objects that need to be added (relevant but not currently replicated)
    let to_add: Vec<ObjectId> = relevant_objects
        .difference(&currently_replicated)
        .cloned()
        .collect();
    
    // Objects that need to be removed (currently replicated but no longer relevant)
    let to_remove: Vec<ObjectId> = currently_replicated
        .difference(&relevant_objects)
        .cloned()
        .collect();
    
    // Add newly relevant objects
    for object_id in to_add {
        ClientObjectReplication::insert_new(
            ctx,
            ClientObjectReplication {
                client_id,
                object_id,
                initial_replication_complete: false,
                last_replication_time: ctx.timestamp().micros(),
            },
        ).map_err(|e| format!("Failed to add new relevant object: {}", e))?;
        
        // In a real implementation, we would trigger an initial replication
        // of this object to the client
    }
    
    // Remove no-longer-relevant objects
    for object_id in to_remove {
        ClientObjectReplication::delete_by_client_id_and_object_id(ctx, client_id, object_id)
            .map_err(|e| format!("Failed to remove no-longer-relevant object: {}", e))?;
        
        // In a real implementation, we would send a message to the client
        // to delete this object locally
    }
    
    Ok(())
}

/// Update last replication time for an object
pub fn update_replication_time(
    ctx: &ReducerContext,
    client_id: u64,
    object_id: ObjectId,
) -> Result<(), String> {
    // Find the entry
    let entries: Vec<_> = ClientObjectReplication::filter_by_client_id_and_object_id(ctx, client_id, object_id).collect();
    
    if entries.is_empty() {
        // Entry doesn't exist, create it
        ClientObjectReplication::insert_new(
            ctx,
            ClientObjectReplication {
                client_id,
                object_id,
                initial_replication_complete: true,
                last_replication_time: ctx.timestamp().micros(),
            },
        ).map_err(|e| format!("Failed to create replication entry: {}", e))?;
    } else {
        // Update the existing entry
        let entry = &entries[0];
        
        // Delete the old entry
        ClientObjectReplication::delete_by_client_id_and_object_id(ctx, client_id, object_id)
            .map_err(|e| format!("Failed to update replication time: {}", e))?;
        
        // Insert the updated entry
        ClientObjectReplication::insert_new(
            ctx,
            ClientObjectReplication {
                client_id,
                object_id,
                initial_replication_complete: true,
                last_replication_time: ctx.timestamp().micros(),
            },
        ).map_err(|e| format!("Failed to update replication time: {}", e))?;
    }
    
    Ok(())
}

/// Get objects that have been replicated to a client
pub fn get_replicated_objects(
    ctx: &ReducerContext,
    client_id: u64,
) -> Vec<ObjectId> {
    ClientObjectReplication::filter_by_client_id(ctx, client_id)
        .map(|entry| entry.object_id)
        .collect()
}

/// Check if an object has been replicated to a client
pub fn is_object_replicated_to_client(
    ctx: &ReducerContext,
    client_id: u64,
    object_id: ObjectId,
) -> bool {
    ClientObjectReplication::filter_by_client_id_and_object_id(ctx, client_id, object_id).next().is_some()
}

/// Clear all replication state for a client (used when a client disconnects)
pub fn clear_client_replication_state(
    ctx: &ReducerContext,
    client_id: u64,
) -> Result<(), String> {
    // Find all entries for this client
    let entries: Vec<_> = ClientObjectReplication::filter_by_client_id(ctx, client_id)
        .map(|entry| entry.object_id)
        .collect();
    
    // Delete each entry
    for object_id in entries {
        ClientObjectReplication::delete_by_client_id_and_object_id(ctx, client_id, object_id)
            .map_err(|e| format!("Failed to clear client replication state: {}", e))?;
    }
    
    Ok(())
} 