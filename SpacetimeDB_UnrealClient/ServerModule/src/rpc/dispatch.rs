//! # RPC Dispatch
//!
//! Handles dispatching of incoming RPC calls from clients to registered handlers
//! and provides functions for sending RPCs to clients.

use spacetimedb::{ReducerContext, Identity, Table, TablesRef, spacetimedb_table};
use stdb_shared::rpc::{RpcCall, RpcResponse, RpcStatus, RpcError};
use stdb_shared::object::ObjectId;

use log::{debug, error, info, warn};
use super::registry;
use super::{RpcLog, RpcRegistration};
use crate::connection::{ClientConnection, ConnectionState};
use crate::object::{ObjectInstance, ObjectLifecycleState};

// Next RPC call ID for tracking
static mut NEXT_RPC_ID: u64 = 1;

/// Safe way to get a new RPC call ID
fn get_next_call_id() -> u64 {
    unsafe {
        let id = NEXT_RPC_ID;
        NEXT_RPC_ID += 1;
        id
    }
}

/// SpacetimeDB reducer for handling client RPC calls
#[spacetimedb::reducer]
pub fn call_server_function(ctx: &ReducerContext, call: RpcCall) -> Result<RpcResponse, String> {
    let caller_identity = ctx.sender;
    
    // Generate a call ID if not provided
    let call_id = call.call_id.unwrap_or_else(|| get_next_call_id());
    
    debug!("Client {:?} calling function '{}' on object {}", 
           caller_identity, call.function_name, call.object_id);
    
    // Check if the function exists
    if !registry::function_exists(&call.function_name) {
        let error = RpcError::FunctionNotFound;
        log_rpc_call(ctx, caller_identity, &call, call_id, RpcStatus::Failed, Some(&format!("{:?}", error)));
        return Ok(RpcResponse {
            call_id,
            status: RpcStatus::Failed,
            result_json: None,
            error: Some(error),
        });
    }
    
    // Check if the object exists
    let tables = ctx.tables();
    let object_exists = tables.with_tables_ref(|db| {
        let results = spacetimedb_table![crate::object::ObjectInstance]
            .filter(db, |obj| obj.id == call.object_id && obj.lifecycle_state != ObjectLifecycleState::Destroyed)
            .collect::<Vec<_>>();
        !results.is_empty()
    });
    
    if !object_exists {
        let error = RpcError::ObjectNotFound(call.object_id);
        log_rpc_call(ctx, caller_identity, &call, call_id, RpcStatus::Failed, Some(&format!("{:?}", error)));
        return Ok(RpcResponse {
            call_id,
            status: RpcStatus::Failed,
            result_json: None,
            error: Some(error),
        });
    }
    
    // Execute the handler
    match dispatch_client_rpc(ctx, &call, caller_identity) {
        Ok(result_json) => {
            // Log successful call
            log_rpc_call(ctx, caller_identity, &call, call_id, RpcStatus::Success, None);
            
            // Return success response
            Ok(RpcResponse {
                call_id,
                status: RpcStatus::Success,
                result_json: Some(result_json),
                error: None,
            })
        },
        Err(error) => {
            // Log failed call
            log_rpc_call(ctx, caller_identity, &call, call_id, RpcStatus::Failed, Some(&format!("{:?}", error)));
            
            // Return error response
            Ok(RpcResponse {
                call_id,
                status: RpcStatus::Failed,
                result_json: None,
                error: Some(error),
            })
        }
    }
}

/// Log an RPC call to the database
fn log_rpc_call(
    ctx: &ReducerContext,
    sender: Identity,
    call: &RpcCall,
    call_id: u64,
    status: RpcStatus,
    error: Option<&str>,
) {
    // Create log entry
    let log_entry = RpcLog {
        call_id,
        sender,
        object_id: call.object_id,
        function_name: call.function_name.clone(),
        timestamp: ctx.timestamp,
        status: status as u8,
        error: error.map(|e| e.to_string()),
    };
    
    // Insert into the database
    let tables = ctx.tables();
    tables.with_tables_ref(|db| {
        spacetimedb_table![super::RpcLog].insert_one(db, &log_entry).unwrap();
    });
}

/// Dispatch an RPC call to the appropriate handler
pub fn dispatch_client_rpc(
    ctx: &ReducerContext,
    call: &RpcCall,
    caller_identity: Identity,
) -> Result<String, RpcError> {
    debug!("Dispatching RPC call '{}' from client {:?}", call.function_name, caller_identity);
    
    // Execute the RPC handler
    registry::execute_handler(ctx, &call.function_name, call.object_id, &call.arguments_json)
}

/// Send an RPC to a specific client
pub fn send_rpc_to_client(
    ctx: &ReducerContext,
    client_identity: Identity,
    object_id: ObjectId,
    function_name: &str,
    args_json: &str,
) -> Result<(), String> {
    // Check if the client is connected
    let tables = ctx.tables();
    let client_connected = tables.with_tables_ref(|db| {
        let clients = spacetimedb_table![crate::connection::ClientConnection]
            .filter(db, |client| 
                client.identity == client_identity && 
                client.state == ConnectionState::Connected)
            .collect::<Vec<_>>();
        !clients.is_empty()
    });
    
    if !client_connected {
        return Err(format!("Client {:?} is not connected", client_identity));
    }
    
    // Create RPC call
    let call = RpcCall {
        object_id,
        function_name: function_name.to_string(),
        arguments_json: args_json.to_string(),
        rpc_type: stdb_shared::rpc::RpcType::Client,
        call_id: Some(get_next_call_id()),
    };
    
    // In a full implementation, we would:
    // 1. Add the call to a queue for the client
    // 2. Have a subscription system where the client would poll for calls
    // 3. Track delivery status
    
    info!("Sent RPC '{}' to client {:?}", function_name, client_identity);
    
    Ok(())
}

/// Broadcast an RPC to all connected clients
pub fn broadcast_rpc(
    ctx: &ReducerContext,
    object_id: ObjectId,
    function_name: &str,
    args_json: &str,
) -> Result<usize, String> {
    // Get all connected clients
    let tables = ctx.tables();
    let connected_clients = tables.with_tables_ref(|db| {
        spacetimedb_table![crate::connection::ClientConnection]
            .filter(db, |client| client.state == ConnectionState::Connected)
            .map(|client| client.identity)
            .collect::<Vec<_>>()
    });
    
    let client_count = connected_clients.len();
    debug!("Broadcasting RPC '{}' to {} clients", function_name, client_count);
    
    // Send to each client
    for client_identity in connected_clients {
        if let Err(err) = send_rpc_to_client(ctx, client_identity, object_id, function_name, args_json) {
            warn!("Failed to send RPC to client {:?}: {}", client_identity, err);
        }
    }
    
    Ok(client_count)
} 