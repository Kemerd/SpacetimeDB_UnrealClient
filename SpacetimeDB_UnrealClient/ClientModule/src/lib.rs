//! # UnrealReplication Client Module
//!
//! This module provides client-side functionality for the SpacetimeDB Unreal Engine integration.
//! It handles property representation, serialization/deserialization, and client-side prediction.
//!
//! The system is organized into several sub-modules:
//! - `property`: Client-side property handling and synchronization
//! - `object`: Client-side UObject representation
//! - `actor`: Client-side actor management
//! - `rpc`: Remote procedure call client implementation
//! - `net`: Network communication with the server

use std::collections::HashMap;

// Module declarations
pub mod property;    // Property system
pub mod object;      // Client-side object representation
pub mod actor;       // Actor-specific functionality
pub mod rpc;         // RPC handling
pub mod net;         // Network layer

// Internal module for FFI (will be extracted to ffi.rs later)
mod ffi;

// Re-export commonly used items
pub use property::{PropertyType, PropertyValue};
pub use object::{ObjectId, ClientObject};
pub use actor::ClientActor;
pub use net::{ConnectionState, ClientConnection};
pub use rpc::RpcResult;

/// Initialize the client module
pub fn init() {
    println!("Initializing UnrealReplication client module");
    
    // Initialize property system
    property::init();
    
    // Initialize object system
    object::init();
    
    // Initialize RPC system
    rpc::init();
    
    println!("UnrealReplication client module initialized!");
}

/// Handle a property update received from the server
pub fn handle_property_update(
    object_id: ObjectId,
    property_name: &str,
    value_json: &str,
) -> Result<(), String> {
    // Deserialize the property value
    let property_value = property::serialization::deserialize_property_value(value_json)?;
    
    // Update the object's property
    object::update_object_property(object_id, property_name, property_value)
}

/// Set a property on an object and send to server if needed
pub fn set_property(
    object_id: ObjectId,
    property_name: &str,
    value: PropertyValue,
    replicate_to_server: bool,
) -> Result<(), String> {
    // First update locally
    object::update_object_property(object_id, property_name, value.clone())?;
    
    // Then replicate to server if needed
    if replicate_to_server {
        let value_json = property::serialization::serialize_property_value(&value)?;
        net::send_property_update(object_id, property_name, &value_json)?;
    }
    
    Ok(())
}

/// Submit an RPC to the server
pub fn call_server_rpc(
    object_id: ObjectId,
    function_name: &str,
    args_json: &str,
) -> Result<(), String> {
    match rpc::call_server_function(object_id, function_name, args_json) {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}

/// Register a client-side RPC handler
pub fn register_client_rpc(
    function_name: &str,
    handler: Box<dyn Fn(ObjectId, &str) -> Result<(), String> + Send + 'static>,
) {
    rpc::register_handler(function_name, handler);
}

/// Handle an incoming RPC call from the server
pub fn handle_server_rpc(
    object_id: ObjectId,
    function_name: &str,
    args_json: &str,
) -> Result<RpcResult, String> {
    rpc::handle_server_call(object_id, function_name, args_json)
}

/// Create a new actor
pub fn create_actor(
    class_name: &str,
    params_json: &str,
) -> Result<ObjectId, String> {
    let params = serde_json::from_str(params_json)
        .map_err(|e| format!("Failed to parse spawn parameters: {}", e))?;
    
    object::create_object(class_name, params)
}

/// Destroy an actor
pub fn destroy_actor(
    object_id: ObjectId,
) -> Result<(), String> {
    object::destroy_object(object_id)
} 