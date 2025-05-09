//! # RPC Module
//! 
//! Handles remote procedure calls between client and server, including function
//! registration, serialization, and execution.

use stdb_shared::object::ObjectId;
use stdb_shared::types::*;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use log::{debug, error, info, warn};

/// Type definition for client-side RPC handlers
pub type ClientRpcHandler = Box<dyn Fn(ObjectId, &str) -> Result<(), String> + Send + 'static>;

/// Registry of client-side RPC handlers
static RPC_HANDLERS: Lazy<Mutex<HashMap<String, ClientRpcHandler>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Structure containing RPC call result
#[derive(Debug, Clone)]
pub struct RpcResult {
    /// Status code of the RPC call
    pub status: StatusCode,
    
    /// Result data (JSON string)
    pub data: Option<String>,
    
    /// Error message if any
    pub error: Option<String>,
}

impl RpcResult {
    /// Create a successful RPC result
    pub fn success(data: Option<String>) -> Self {
        Self {
            status: StatusCode::Success,
            data,
            error: None,
        }
    }
    
    /// Create a failed RPC result
    pub fn failure(error_msg: &str, status: StatusCode) -> Self {
        Self {
            status,
            data: None,
            error: Some(error_msg.to_string()),
        }
    }
}

/// Initialize the RPC system
pub fn init() {
    info!("Initializing RPC system");
    
    // Register built-in RPC handlers
    register_builtin_handlers();
    
    info!("RPC system initialized");
}

/// Register a client-side RPC handler function
pub fn register_handler(function_name: &str, handler: ClientRpcHandler) {
    let mut handlers = RPC_HANDLERS.lock().unwrap();
    
    if handlers.contains_key(function_name) {
        warn!("Replacing existing RPC handler for function: {}", function_name);
    }
    
    handlers.insert(function_name.to_string(), handler);
    debug!("Registered RPC handler for function: {}", function_name);
}

/// Unregister a client-side RPC handler
pub fn unregister_handler(function_name: &str) {
    let mut handlers = RPC_HANDLERS.lock().unwrap();
    handlers.remove(function_name);
    debug!("Unregistered RPC handler for function: {}", function_name);
}

/// Call a server RPC function
pub fn call_server_function(
    object_id: ObjectId,
    function_name: &str,
    args_json: &str,
) -> Result<RpcResult, String> {
    // Get the client ID
    let client_id = crate::net::get_client_id();
    
    // Check if connected
    if !crate::net::is_connected() {
        return Err("Not connected to server".to_string());
    }
    
    info!("Calling server function: {}", function_name);
    debug!("  Object ID: {}", object_id);
    debug!("  Arguments: {}", args_json);
    
    // Create a call signature for logging/debugging
    let call_signature = format!("{}({}) on Object {}", function_name, args_json, object_id);
    
    // Use the network module to send the RPC call to the server via SpacetimeDB SDK
    match send_rpc_to_server(client_id, object_id, function_name, args_json) {
        Ok(_) => {
            info!("Successfully called: {}", call_signature);
            Ok(RpcResult::success(None))
        },
        Err(err) => {
            error!("Failed to call {}: {}", call_signature, err);
            Ok(RpcResult::failure(&err, StatusCode::NetworkError))
        }
    }
}

/// Handle an incoming RPC call from the server
pub fn handle_server_call(
    object_id: ObjectId,
    function_name: &str,
    args_json: &str,
) -> Result<RpcResult, String> {
    info!("Received server RPC call: {}", function_name);
    debug!("  Object ID: {}", object_id);
    debug!("  Arguments: {}", args_json);
    
    // Look up the handler
    let handlers = RPC_HANDLERS.lock().unwrap();
    
    if let Some(handler) = handlers.get(function_name) {
        // Execute the handler
        match handler(object_id, args_json) {
            Ok(()) => {
                debug!("Successfully executed RPC handler for: {}", function_name);
                Ok(RpcResult::success(None))
            },
            Err(err) => {
                warn!("RPC handler execution failed for {}: {}", function_name, err);
                Ok(RpcResult::failure(&err, StatusCode::Failure))
            }
        }
    } else {
        let error_msg = format!("No handler registered for RPC function: {}", function_name);
        warn!("{}", error_msg);
        Ok(RpcResult::failure(&error_msg, StatusCode::NotFound))
    }
}

/// Register built-in RPC handlers
fn register_builtin_handlers() {
    // Register ping handler for connection testing
    register_handler("ping", Box::new(|_object_id, _args| {
        debug!("Received ping, sending pong");
        Ok(())
    }));
    
    // Register echo handler for testing
    register_handler("echo", Box::new(|_object_id, args| {
        debug!("Echo: {}", args);
        Ok(())
    }));
}

/// Send an RPC call to the server
fn send_rpc_to_server(
    client_id: u64,
    object_id: ObjectId,
    function_name: &str,
    args_json: &str,
) -> Result<(), String> {
    // Build the RPC request message
    let rpc_request = format!(
        r#"{{
            "client_id": {},
            "object_id": {},
            "function": "{}",
            "args": {}
        }}"#,
        client_id, object_id, function_name, args_json
    );
    
    // Use the SpacetimeDB SDK via our network module to call the server's execute_rpc reducer
    // This is the actual implementation that uses the SDK correctly
    crate::net::send_rpc_request(&rpc_request)
}

/// Register a callback for when the server calls this function
pub fn register_callback<F>(function_name: &str, callback: F)
where
    F: Fn(ObjectId, &str) -> Result<(), String> + Send + 'static,
{
    register_handler(function_name, Box::new(callback));
}

/// Create a response to a server RPC call
pub fn create_response(data: Option<&str>) -> String {
    match data {
        Some(data_str) => format!(r#"{{"status": "success", "data": {}}}"#, data_str),
        None => r#"{"status": "success"}"#.to_string(),
    }
}

/// Create an error response to a server RPC call
pub fn create_error_response(error_msg: &str) -> String {
    format!(r#"{{"status": "error", "message": "{}"}}"#, error_msg)
} 