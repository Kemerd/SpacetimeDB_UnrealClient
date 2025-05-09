//! # Connection Handlers
//!
//! Handles client connection and disconnection events.

use spacetimedb::{ReducerContext, TableType, Identity};
use serde_json;
use crate::connection::{ClientInfo, generate_client_id};
use crate::SharedModule::connection::{ConnectionParams, DisconnectReason};

/// Register a new client when they connect to the server
pub fn register_client(ctx: &ReducerContext) {
    if ctx.sender.is_none() {
        log::warn!("Attempted to register client with no identity");
        return;
    }

    let identity = ctx.sender.unwrap();
    
    // Check if client already exists (reconnection case)
    let existing_client = ctx.db.client_info().filter_by_identity(&identity).first();
    
    if let Some(mut client) = existing_client {
        // Update existing client record
        log::info!("Client reconnected: {:?} (ID: {})", identity, client.client_id);
        client.connected_at = ctx.timestamp;
        client.last_activity = ctx.timestamp;
        ctx.db.client_info().update(&client);
    } else {
        // Create new client record
        let client_id = generate_client_id();
        log::info!("New client connected: {:?} (assigned ID: {})", identity, client_id);
        
        // Try to extract connection parameters from client context
        // In a real implementation, clients would send these as part of their initial connection
        let conn_params = None; // Default to None, would be extracted from client message in real impl
        
        // Insert the new client
        ctx.db.client_info().insert(ClientInfo {
            identity,
            client_id,
            display_name: None, // Client can set this later
            is_admin: false,    // Default to non-admin, can be changed later based on auth
            connected_at: ctx.timestamp,
            connection_params: conn_params.map(|p| serde_json::to_string(&p).unwrap_or_default()),
            last_activity: ctx.timestamp,
        });
    }
    
    // Notify other server modules that a client has connected
    // This could be done via a publish/subscribe mechanism in a full implementation
    log::debug!("Broadcasting client connection event");
}

/// Unregister a client when they disconnect from the server
pub fn unregister_client(ctx: &ReducerContext) {
    if ctx.sender.is_none() {
        log::warn!("Attempted to unregister client with no identity");
        return;
    }

    let identity = ctx.sender.unwrap();
    
    // Find the client in our database
    let client = match ctx.db.client_info().filter_by_identity(&identity).first() {
        Some(client) => client,
        None => {
            log::warn!("Client disconnected but wasn't registered: {:?}", identity);
            return;
        }
    };
    
    log::info!("Client disconnected: {:?} (ID: {})", identity, client.client_id);
    
    // Clean up client-owned actors
    cleanup_client_actors(ctx, identity);
    
    // We have two options here:
    // 1. Remove the client record completely (which we do here)
    // 2. Mark them as disconnected but keep their record (for later reconnection)
    //    This would require adding a 'connected' bool field to ClientInfo
    
    ctx.db.client_info().delete_by_identity(&identity);
    
    // Notify other server modules that a client has disconnected
    log::debug!("Broadcasting client disconnection event");
}

/// Update a client's display name
#[spacetimedb::reducer]
pub fn set_client_display_name(ctx: &ReducerContext, display_name: String) -> bool {
    if ctx.sender.is_none() {
        log::warn!("Attempted to set display name with no identity");
        return false;
    }

    let identity = ctx.sender.unwrap();
    
    // Find the client in our database
    let client = match ctx.db.client_info().filter_by_identity(&identity).first() {
        Some(client) => client,
        None => {
            log::warn!("Client not found: {:?}", identity);
            return false;
        }
    };
    
    // Update display name
    let mut updated_client = client.clone();
    updated_client.display_name = Some(display_name.clone());
    updated_client.last_activity = ctx.timestamp;
    ctx.db.client_info().update(&updated_client);
    
    log::info!("Client {:?} (ID: {}) set display name to '{}'", 
              identity, client.client_id, display_name);
    
    true
}

/// Process connection parameters sent by the client
#[spacetimedb::reducer]
pub fn set_connection_params(ctx: &ReducerContext, params_json: String) -> bool {
    if ctx.sender.is_none() {
        log::warn!("Attempted to set connection params with no identity");
        return false;
    }

    let identity = ctx.sender.unwrap();
    
    // Find the client in our database
    let client = match ctx.db.client_info().filter_by_identity(&identity).first() {
        Some(client) => client,
        None => {
            log::warn!("Client not found: {:?}", identity);
            return false;
        }
    };
    
    // Parse connection parameters
    let params_result: Result<ConnectionParams, _> = serde_json::from_str(&params_json);
    if let Err(e) = params_result {
        log::warn!("Invalid connection params from client {:?}: {}", identity, e);
        return false;
    }
    
    // Update client record
    let mut updated_client = client.clone();
    updated_client.connection_params = Some(params_json);
    updated_client.last_activity = ctx.timestamp;
    ctx.db.client_info().update(&updated_client);
    
    log::debug!("Updated connection params for client {:?}", identity);
    
    true
}

/// Helper function to clean up actors owned by a disconnected client
fn cleanup_client_actors(ctx: &ReducerContext, identity: Identity) {
    // Find all actors owned by this client
    let owned_actors: Vec<_> = ctx.db.actor_info()
        .iter()
        .filter(|actor| actor.owner_identity == Some(identity))
        .collect();
    
    if owned_actors.is_empty() {
        return;
    }
    
    log::info!("Cleaning up {} actors owned by disconnected client {:?}", 
              owned_actors.len(), identity);
    
    for actor in owned_actors {
        // Mark actors as pending destroy
        let mut updated_actor = actor.clone();
        updated_actor.state = crate::actor::ActorLifecycleState::PendingDestroy;
        ctx.db.actor_info().update(&updated_actor);
    }
} 