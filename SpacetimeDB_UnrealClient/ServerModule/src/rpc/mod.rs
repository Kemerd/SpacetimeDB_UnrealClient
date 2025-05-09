//! # RPC Module (Server-Side)
//! 
//! Provides server-side RPC functionality including:
//! - RPC function registration mechanism
//! - Logic for dispatching incoming RPC calls from clients
//! - Functions for sending RPCs from server to client(s)

use spacetimedb::{ReducerContext, Identity, TablesRef, SpacetimeType, TableType, Table};
use stdb_shared::rpc::{RpcCall, RpcResponse, RpcType, RpcStatus, RpcError, RpcFunctionInfo};
use stdb_shared::object::ObjectId;
use stdb_shared::types::*;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use once_cell::sync::Lazy;
use log::{debug, error, info, warn};

use crate::connection::{ClientConnection, ConnectionState};

// Export public types and functions
pub use registry::{register_rpc, get_registered_functions, register_module_rpcs};
pub use dispatch::{dispatch_client_rpc, send_rpc_to_client, broadcast_rpc};

/// Type definition for server-side RPC handlers
pub type ServerRpcHandler = Arc<dyn Fn(&ReducerContext, ObjectId, &str) -> Result<String, RpcError> + Send + Sync + 'static>;

// Module for RPC registration and function registry
pub mod registry;

// Module for dispatching RPC calls
pub mod dispatch;

// Module for sending RPCs to clients
pub mod outgoing;

// Define RPC-related SpacetimeDB tables
#[derive(SpacetimeType, Clone, Debug)]
#[spacetimedb(table)]
pub struct RpcRegistration {
    #[primarykey]
    pub name: String,
    pub class_name: String,
    pub rpc_type: u8,
    pub is_reliable: bool,
}

#[derive(SpacetimeType, Clone, Debug)]
#[spacetimedb(table)]
pub struct RpcLog {
    #[primarykey]
    pub call_id: u64,
    pub sender: Identity,
    pub object_id: ObjectId,
    pub function_name: String,
    pub timestamp: u64,
    pub status: u8,
    pub error: Option<String>,
}

// Initialize the RPC system
pub fn init(ctx: &ReducerContext) {
    info!("Initializing RPC system");
    
    // Register built-in RPCs
    register_builtin_rpcs();
    
    info!("RPC system initialized");
}

// Register built-in RPC functions
fn register_builtin_rpcs() {
    // Register ping handler for connection testing
    registry::register_rpc(
        "ping", 
        "Global", 
        RpcType::Server,
        true,
        Arc::new(|ctx, object_id, _args| {
            debug!("Received ping from client {:?}, sending pong", ctx.sender);
            Ok("\"pong\"".to_string())
        })
    );
    
    // Register echo handler for testing
    registry::register_rpc(
        "echo", 
        "Global", 
        RpcType::Server,
        true,
        Arc::new(|ctx, object_id, args| {
            debug!("Echo from client {:?}: {}", ctx.sender, args);
            Ok(args.to_string())
        })
    );
} 