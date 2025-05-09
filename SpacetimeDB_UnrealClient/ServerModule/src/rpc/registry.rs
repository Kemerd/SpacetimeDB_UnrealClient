//! # RPC Registry
//!
//! Manages registration of RPC functions and provides lookup capabilities

use stdb_shared::rpc::{RpcType, RpcError, RpcFunctionInfo};
use stdb_shared::object::ObjectId;
use spacetimedb::ReducerContext;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use once_cell::sync::Lazy;
use log::{debug, error, info, warn};

use super::ServerRpcHandler;

/// Global registry of RPC functions with their handlers
static RPC_REGISTRY: Lazy<RwLock<HashMap<String, (RpcFunctionInfo, ServerRpcHandler)>>> = 
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Register a new RPC function with a handler
pub fn register_rpc(
    name: &str,
    class_name: &str,
    rpc_type: RpcType,
    is_reliable: bool,
    handler: ServerRpcHandler,
) {
    let function_info = RpcFunctionInfo {
        name: name.to_string(),
        class_name: class_name.to_string(),
        rpc_type,
        is_reliable,
    };
    
    let mut registry = RPC_REGISTRY.write().unwrap();
    
    if registry.contains_key(name) {
        warn!("Replacing existing RPC registration for function: {}", name);
    }
    
    registry.insert(name.to_string(), (function_info, handler));
    debug!("Registered RPC function: {}", name);
}

/// Get a list of all registered RPC functions
pub fn get_registered_functions() -> Vec<RpcFunctionInfo> {
    let registry = RPC_REGISTRY.read().unwrap();
    registry.values().map(|(info, _)| info.clone()).collect()
}

/// Look up a handler for a given function name
pub fn get_handler(function_name: &str) -> Option<ServerRpcHandler> {
    let registry = RPC_REGISTRY.read().unwrap();
    registry.get(function_name).map(|(_, handler)| handler.clone())
}

/// Get function info for a given function name
pub fn get_function_info(function_name: &str) -> Option<RpcFunctionInfo> {
    let registry = RPC_REGISTRY.read().unwrap();
    registry.get(function_name).map(|(info, _)| info.clone())
}

/// Check if a function exists in the registry
pub fn function_exists(function_name: &str) -> bool {
    let registry = RPC_REGISTRY.read().unwrap();
    registry.contains_key(function_name)
}

/// Register multiple RPC functions from a module
pub fn register_module_rpcs<F>(module_name: &str, register_fn: F)
where
    F: FnOnce(&str),
{
    info!("Registering RPCs for module: {}", module_name);
    register_fn(module_name);
}

/// Unregister an RPC function
pub fn unregister_rpc(function_name: &str) {
    let mut registry = RPC_REGISTRY.write().unwrap();
    if registry.remove(function_name).is_some() {
        debug!("Unregistered RPC function: {}", function_name);
    } else {
        warn!("Attempted to unregister non-existent RPC function: {}", function_name);
    }
}

/// Clear all registered RPC functions
pub fn clear_registry() {
    let mut registry = RPC_REGISTRY.write().unwrap();
    registry.clear();
    debug!("Cleared RPC registry");
}

/// Execute an RPC handler with proper error handling
pub fn execute_handler(
    ctx: &ReducerContext,
    function_name: &str,
    object_id: ObjectId,
    args_json: &str,
) -> Result<String, RpcError> {
    // Look up the handler
    if let Some(handler) = get_handler(function_name) {
        // Execute the handler and capture any panics
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            handler(ctx, object_id, args_json)
        })) {
            Ok(result) => result,
            Err(_) => {
                let error_msg = format!("RPC handler for '{}' panicked during execution", function_name);
                error!("{}", error_msg);
                Err(RpcError::ExecutionFailed(error_msg))
            }
        }
    } else {
        Err(RpcError::FunctionNotFound)
    }
} 