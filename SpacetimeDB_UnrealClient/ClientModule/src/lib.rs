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
//! - `prediction`: Client-side prediction handling
//! - `class`: Class handling

use std::collections::HashMap;

// Module declarations
pub mod property;    // Property system
pub mod object;      // Client-side object representation
pub mod actor;       // Actor-specific functionality
pub mod rpc;         // RPC handling
pub mod net;         // Network layer
pub mod prediction;  // Prediction handling
pub mod class;       // Class handling

// Internal module for FFI (will be extracted to ffi.rs later)
mod ffi;

// Re-export commonly used items
pub use property::{
    get_property_definition, get_property_definition_count, 
    has_property_definitions_for_class, get_property_names_for_class,
    get_registered_class_names, import_property_definitions_from_json, 
    export_property_definitions_as_json
};
// Re-export types directly from stdb_shared to avoid private re-exports
pub use stdb_shared::property::{PropertyType, PropertyValue, ReplicationCondition};
pub use object::ClientObject;
pub use stdb_shared::object::ObjectId;
pub use stdb_shared::connection::{ConnectionState, ClientConnection};
pub use rpc::RpcResult;

/// Initialize the client module
pub fn init() {
    println!("Initializing UnrealReplication client module");
    
    // Initialize property system
    property::init();
    
    // Initialize object system
    object::init();
    
    // Initialize actor system
    actor::init();
    
    // Initialize RPC system
    rpc::init();
    
    // Initialize prediction system
    prediction::init();
    
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

// Provide the missing WebAssembly host functions that are used by the SpacetimeDB library
// These are normally provided by the SpacetimeDB runtime in a WebAssembly environment

#[no_mangle]
pub extern "C" fn bytes_source_read(_ptr: *mut u8, _len: usize) -> u32 {
    // Stub implementation
    0
}

#[no_mangle]
pub extern "C" fn bytes_sink_write(_ptr: *const u8, _len: usize) {
    // Stub implementation
}

#[no_mangle]
pub extern "C" fn console_log(_level: u8, _target: *const u8, _target_len: usize, 
                            _filename: *const u8, _filename_len: usize,
                            _line: u32, _message: *const u8, _message_len: usize) {
    // Stub implementation
}

#[no_mangle]
pub extern "C" fn identity() -> [u8; 32] {
    // Return empty identity
    [0; 32]
}

#[no_mangle]
pub extern "C" fn table_id_from_name(_name: *const u8, _name_len: usize) -> u32 {
    // Stub implementation
    0
}

#[no_mangle]
pub extern "C" fn console_timer_start(_name: *const u8, _name_len: usize) -> u32 {
    // Stub implementation
    0
}

#[no_mangle]
pub extern "C" fn console_timer_end(_timer_id: u32) {
    // Stub implementation
} 