//! # Connection Management
//! 
//! This module handles client connections to the SpacetimeDB instance,
//! including client registration, authentication, and disconnection.

use spacetimedb::{Identity, ReducerContext, TableType};
use crate::SharedModule::connection::{ConnectionState, ClientIdentity, ConnectionParams, ClientConnection, DisconnectReason};

// Module declarations
pub mod handlers;  // Client connection event handlers
pub mod auth;      // Authentication and authorization

// Tables
// Store information about connected clients
#[spacetimedb::table]
pub struct ClientInfo {
    #[primarykey]
    pub identity: Identity,
    pub client_id: u64,
    pub display_name: Option<String>,
    pub is_admin: bool,
    pub connected_at: u64,
    pub connection_params: Option<String>, // Serialized ConnectionParams
    pub last_activity: u64,
}

// Counter for generating unique client IDs
static mut NEXT_CLIENT_ID: u64 = 1000; // Start at 1000 to leave room for special IDs

/// Generates a unique client ID
pub fn generate_client_id() -> u64 {
    unsafe {
        let id = NEXT_CLIENT_ID;
        NEXT_CLIENT_ID += 1;
        id
    }
}

// Re-exports for use in other modules
pub use handlers::{register_client, unregister_client};
pub use auth::{can_spawn_actor, is_admin}; 