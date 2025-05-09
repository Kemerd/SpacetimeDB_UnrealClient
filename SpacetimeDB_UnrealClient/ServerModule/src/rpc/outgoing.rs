//! # Outgoing RPC Module
//!
//! Specializes in sending RPCs from server to client(s) with various targeting options
//! including multicast, owner-only, and specific client groups.

use spacetimedb::{ReducerContext, Identity};
use stdb_shared::object::ObjectId;
use stdb_shared::rpc::RpcType;

use log::{debug, error, info, warn};
use super::dispatch;
use crate::object::{ObjectInstance, ObjectLifecycleState};
use crate::connection::{ClientConnection, ConnectionState};

/// Send an RPC to the owner of an object
pub fn send_rpc_to_owner(
    ctx: &ReducerContext,
    object_id: ObjectId,
    function_name: &str,
    args_json: &str,
) -> Result<(), String> {
    // Find the owner of the object
    let tables = ctx.tables();
    let owner_identity = tables.with_tables_ref(|db| {
        let objects = spacetimedb::spacetimedb_table![crate::object::ObjectInstance]
            .filter(db, |obj| obj.id == object_id && obj.lifecycle_state != ObjectLifecycleState::Destroyed)
            .collect::<Vec<_>>();
        
        if objects.is_empty() {
            return None;
        }
        
        let object = &objects[0];
        Some(object.owner)
    });
    
    // If we found an owner, send the RPC to them
    if let Some(owner) = owner_identity {
        dispatch::send_rpc_to_client(ctx, owner, object_id, function_name, args_json)
    } else {
        Err(format!("No owner found for object {}", object_id))
    }
}

/// Send an RPC to all clients in a relevancy zone
pub fn send_rpc_to_zone(
    ctx: &ReducerContext,
    zone_id: u64,
    object_id: ObjectId,
    function_name: &str,
    args_json: &str,
) -> Result<usize, String> {
    // In a full implementation, we would query relevancy zones and find clients
    // For now, we'll just broadcast to all clients with a warning
    warn!("Zone-based RPC not fully implemented; broadcasting to all clients");
    dispatch::broadcast_rpc(ctx, object_id, function_name, args_json)
}

/// Send an RPC to clients who have the object in their relevancy set
pub fn send_rpc_to_relevant_clients(
    ctx: &ReducerContext,
    object_id: ObjectId,
    function_name: &str,
    args_json: &str,
) -> Result<usize, String> {
    // In a full implementation, we would query which clients consider
    // this object relevant based on distance, visibility, etc.
    // For now, we'll just broadcast to all clients with a warning
    warn!("Relevancy-based RPC not fully implemented; broadcasting to all clients");
    dispatch::broadcast_rpc(ctx, object_id, function_name, args_json)
}

/// Send an RPC to a group of clients
pub fn send_rpc_to_group(
    ctx: &ReducerContext,
    group_name: &str,
    object_id: ObjectId,
    function_name: &str,
    args_json: &str,
) -> Result<usize, String> {
    // In a full implementation, we would have a concept of client groups
    // For now, we just warn and broadcast to all
    warn!("Group-based RPC not fully implemented; broadcasting to all clients");
    dispatch::broadcast_rpc(ctx, object_id, function_name, args_json)
}

/// Send an RPC to all clients except one
pub fn send_rpc_to_others(
    ctx: &ReducerContext,
    excluded_identity: Identity,
    object_id: ObjectId,
    function_name: &str,
    args_json: &str,
) -> Result<usize, String> {
    // Get all connected clients except the excluded one
    let tables = ctx.tables();
    let connected_clients = tables.with_tables_ref(|db| {
        spacetimedb::spacetimedb_table![crate::connection::ClientConnection]
            .filter(db, |client| 
                client.state == ConnectionState::Connected && 
                client.identity != excluded_identity)
            .map(|client| client.identity)
            .collect::<Vec<_>>()
    });
    
    let client_count = connected_clients.len();
    debug!("Sending RPC '{}' to {} clients (excluding {:?})", 
           function_name, client_count, excluded_identity);
    
    // Send to each client
    let mut success_count = 0;
    for client_identity in connected_clients {
        if let Err(err) = dispatch::send_rpc_to_client(ctx, client_identity, object_id, function_name, args_json) {
            warn!("Failed to send RPC to client {:?}: {}", client_identity, err);
        } else {
            success_count += 1;
        }
    }
    
    Ok(success_count)
} 