//! # Authentication and Authorization
//!
//! Provides functions for authenticating clients and authorizing actions.

use spacetimedb::{ReducerContext, TableType, Identity};
use std::collections::HashMap;
use once_cell::sync::Lazy;

// Simple mapping of class IDs to required permissions
// In a real implementation, this would likely be loaded from configuration
static CLASS_PERMISSIONS: Lazy<HashMap<u32, &'static str>> = Lazy::new(|| {
    let mut map = HashMap::new();
    // Default permissions for common classes
    map.insert(1, "actor.spawn.basic");       // Basic Actor
    map.insert(2, "actor.spawn.pawn");        // Pawn
    map.insert(3, "actor.spawn.character");   // Character
    map.insert(4, "actor.spawn.player");      // PlayerController
    map.insert(5, "actor.spawn.weapon");      // Weapon
    // Add more as needed
    map
});

/// Check if a client is authorized to spawn an actor of the given class
pub fn can_spawn_actor(ctx: &ReducerContext, class_id: u32) -> bool {
    // System calls can always spawn actors
    if ctx.sender.is_none() {
        return true;
    }

    let identity = ctx.sender.unwrap();
    
    // Admins can spawn any actor
    if is_admin_by_identity(ctx, identity) {
        return true;
    }
    
    // Find the client
    let client = match ctx.db.client_info().filter_by_identity(&identity).first() {
        Some(client) => client,
        None => {
            log::warn!("Client not found in can_spawn_actor check: {:?}", identity);
            return false;
        }
    };
    
    // In a real implementation, we would check if the client has the required permission
    // for the specific class_id. For now, we'll use a simplistic approach:
    
    // Check if there's a specific permission required for this class
    if let Some(required_permission) = CLASS_PERMISSIONS.get(&class_id) {
        // Here we would check if the client has this permission
        // For now, we'll just allow basic spawning for connected clients
        if *required_permission == "actor.spawn.basic" {
            return true;
        }
        
        // Allow pawns and characters for most clients
        if *required_permission == "actor.spawn.pawn" || *required_permission == "actor.spawn.character" {
            return true;
        }
        
        // Restrict more privileged classes
        if *required_permission == "actor.spawn.player" || *required_permission == "actor.spawn.weapon" {
            // In a real implementation, we'd check specific permissions
            // For now, only allow admins (handled above)
            return false;
        }
    }
    
    // Default to permissive for unspecified classes
    // In a production environment, you'd likely want this to be false
    true
}

/// Check if the context represents an admin client
pub fn is_admin(ctx: &ReducerContext) -> bool {
    // System calls are always admin-level
    if ctx.sender.is_none() {
        return true;
    }

    let identity = ctx.sender.unwrap();
    is_admin_by_identity(ctx, identity)
}

/// Helper to check if a specific identity has admin status
fn is_admin_by_identity(ctx: &ReducerContext, identity: Identity) -> bool {
    // Find the client
    let client = match ctx.db.client_info().filter_by_identity(&identity).first() {
        Some(client) => client,
        None => {
            log::warn!("Client not found in admin check: {:?}", identity);
            return false;
        }
    };
    
    // Check if the client has admin flag
    client.is_admin
}

/// Set a client's admin status (admin-only operation)
#[spacetimedb::reducer]
pub fn set_client_admin_status(ctx: &ReducerContext, target_identity: Identity, is_admin: bool) -> bool {
    // Only admins can set admin status
    if !is_admin(ctx) {
        log::warn!("Non-admin attempted to change admin status: {:?}", ctx.sender);
        return false;
    }
    
    // Find the target client
    let client = match ctx.db.client_info().filter_by_identity(&target_identity).first() {
        Some(client) => client,
        None => {
            log::warn!("Target client not found: {:?}", target_identity);
            return false;
        }
    };
    
    // Update admin status
    let mut updated_client = client.clone();
    updated_client.is_admin = is_admin;
    ctx.db.client_info().update(&updated_client);
    
    log::info!("Client {:?} admin status set to {} by {:?}", 
              target_identity, is_admin, ctx.sender);
    
    true
}

/// Check if a client has a specific permission
/// In a real implementation, this would use a more sophisticated permission system
pub fn has_permission(ctx: &ReducerContext, permission: &str) -> bool {
    // Admins have all permissions
    if is_admin(ctx) {
        return true;
    }
    
    if ctx.sender.is_none() {
        return false;
    }
    
    let identity = ctx.sender.unwrap();
    
    // Find the client
    let client = match ctx.db.client_info().filter_by_identity(&identity).first() {
        Some(client) => client,
        None => {
            log::warn!("Client not found in permission check: {:?}", identity);
            return false;
        }
    };
    
    // In a real implementation, we would have a permissions table and check if the client
    // has the specific permission. For now, we'll just implement some basic rules.
    
    match permission {
        // Basic permissions that all authenticated clients have
        "connection.basic" => true,
        "actor.view" => true,
        "property.read" => true,
        
        // More restricted permissions
        "actor.spawn.basic" => true,  // Allow basic actor spawning for all clients
        "actor.spawn.pawn" => true,   // Allow pawn spawning for all clients
        "actor.spawn.character" => true, // Allow character spawning for all clients
        "property.write" => true,     // Allow property writing for all clients
        
        // Very restricted permissions (admin only, handled above)
        "actor.spawn.player" => false,
        "actor.spawn.weapon" => false,
        "admin.commands" => false,
        
        // Unknown permission - default to denial
        _ => false,
    }
} 