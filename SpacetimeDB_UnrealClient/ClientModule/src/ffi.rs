//! # FFI Layer
//!
//! Foreign Function Interface layer that bridges between Rust and C++ code.
//! This provides the integration between our Rust client module and Unreal Engine.

use cxx::{CxxString, CxxVector};
use std::sync::{Arc, Mutex};
use std::str::FromStr;
use std::collections::HashMap;
use once_cell::sync::Lazy;

use stdb_shared::object::ObjectId;
use stdb_shared::property::{PropertyType, PropertyValue};
use stdb_shared::types::*;

use crate::property;
use crate::object;
use crate::net;
use crate::rpc;

// --- Global state for FFI callbacks ---

/// Callback function pointers for C++ integration
#[derive(Debug, Default)]
struct EventCallbacks {
    on_connected: usize,
    on_disconnected: usize,
    on_property_updated: usize,
    on_object_created: usize,
    on_object_destroyed: usize,
    on_error_occurred: usize,
    on_object_id_remapped: usize,
}

/// Global storage for callback function pointers
static CALLBACKS: Lazy<Mutex<EventCallbacks>> = Lazy::new(|| Mutex::new(EventCallbacks::default()));

// --- Helper functions for callback invocation ---
// These functions safely convert `usize` back to function pointers and call them.
// They are marked `unsafe` because they dereference raw pointers.
use std::ffi::{c_char, CStr, CString};

fn invoke_on_connected(cb_ptr: usize) {
    if cb_ptr != 0 {
        let func: unsafe extern "C" fn() = unsafe { std::mem::transmute(cb_ptr) };
        unsafe { func() };
    }
}

fn invoke_on_disconnected(cb_ptr: usize, reason: &str) {
    if cb_ptr != 0 {
        let func: unsafe extern "C" fn(*const c_char) = unsafe { std::mem::transmute(cb_ptr) };
        let c_reason = CString::new(reason).unwrap_or_else(|_| CString::new("unknown reason").unwrap());
        unsafe { func(c_reason.as_ptr()) };
    }
}

fn invoke_on_property_updated(cb_ptr: usize, object_id: ObjectId, property_name: &str, value_json: &str) {
    if cb_ptr != 0 {
        let func: unsafe extern "C" fn(u64, *const c_char, *const c_char) = 
            unsafe { std::mem::transmute(cb_ptr) };
        
        let c_property_name = CString::new(property_name)
            .unwrap_or_else(|_| CString::new("unknown").unwrap());
        
        let c_value_json = CString::new(value_json)
            .unwrap_or_else(|_| CString::new("{}").unwrap());
        
        unsafe { func(object_id, c_property_name.as_ptr(), c_value_json.as_ptr()) };
    }
}

fn invoke_on_object_created(cb_ptr: usize, object_id: ObjectId, class_name: &str, data_json: &str) {
    if cb_ptr != 0 {
        let func: unsafe extern "C" fn(u64, *const c_char, *const c_char) = 
            unsafe { std::mem::transmute(cb_ptr) };
        
        let c_class_name = CString::new(class_name)
            .unwrap_or_else(|_| CString::new("unknown").unwrap());
        
        let c_data_json = CString::new(data_json)
            .unwrap_or_else(|_| CString::new("{}").unwrap());
        
        unsafe { func(object_id, c_class_name.as_ptr(), c_data_json.as_ptr()) };
    }
}

fn invoke_on_object_destroyed(cb_ptr: usize, object_id: ObjectId) {
    if cb_ptr != 0 {
        let func: unsafe extern "C" fn(u64) = unsafe { std::mem::transmute(cb_ptr) };
        unsafe { func(object_id) };
    }
}

/// Invoke the callback for when an object ID is remapped from temporary to server-assigned
pub fn invoke_on_object_id_remapped(temp_id: ObjectId, server_id: ObjectId) {
    let cb_ptr = CALLBACKS.lock().unwrap().on_object_id_remapped;
    if cb_ptr != 0 {
        let func: unsafe extern "C" fn(u64, u64) = unsafe { std::mem::transmute(cb_ptr) };
        unsafe { func(temp_id, server_id) };
    }
}

fn invoke_on_error(cb_ptr: usize, error_message: &str) {
    if cb_ptr != 0 {
        let func: unsafe extern "C" fn(*const c_char) = unsafe { std::mem::transmute(cb_ptr) };
        let c_error_message = CString::new(error_message)
            .unwrap_or_else(|_| CString::new("unknown error").unwrap());
        unsafe { func(c_error_message.as_ptr()) };
    }
}

// --- FFI Bridge Definition ---
// This section uses `cxx::bridge` to define the interface between Rust and C++.
#[cxx::bridge(namespace = "stdb::ffi")]
mod bridge {
    // Configuration for connecting to SpacetimeDB.
    struct ConnectionConfig {
        host: String,
        db_name: String,
        auth_token: String, 
    }

    // Struct to pass callback function pointers from C++ to Rust.
    // `usize` is used to represent raw function pointers.
    #[derive(Debug, Default)]
    struct EventCallbackPointers {
        on_connected: usize,
        on_disconnected: usize,
        on_property_updated: usize,
        on_object_created: usize,
        on_object_destroyed: usize,
        on_error_occurred: usize,
        on_object_id_remapped: usize,
    }

    // Functions exposed from Rust to C++.
    extern "Rust" {
        // Connection management
        fn connect_to_server(config: ConnectionConfig, callbacks: EventCallbackPointers) -> bool;
        fn disconnect_from_server() -> bool;
        fn is_connected() -> bool;
        
        // Property management
        fn set_property(object_id: u64, property_name: &CxxString, value_json: &CxxString, replicate: bool) -> bool;
        fn get_property(object_id: u64, property_name: &CxxString) -> CxxString;
        
        // Object management
        fn create_object(class_name: &CxxString, params_json: &CxxString) -> u64;
        fn destroy_object(object_id: u64) -> bool;
        
        // RPC (Remote Procedure Calls)
        fn call_server_function(object_id: u64, function_name: &CxxString, args_json: &CxxString) -> bool;
        fn register_client_function(function_name: &CxxString, handler_ptr: usize) -> bool;
        
        // Utility functions
        fn get_client_id() -> u64;
        fn get_object_class(object_id: u64) -> CxxString;
    }
}

// --- FFI Implementation ---

/// Connect to the SpacetimeDB server
fn connect_to_server(config: bridge::ConnectionConfig, callbacks: bridge::EventCallbackPointers) -> bool {
    println!("Connecting to SpacetimeDB at {}/{}", config.host, config.db_name);
    
    // Store callback pointers
    {
        let mut cb = CALLBACKS.lock().unwrap();
        cb.on_connected = callbacks.on_connected;
        cb.on_disconnected = callbacks.on_disconnected;
        cb.on_property_updated = callbacks.on_property_updated;
        cb.on_object_created = callbacks.on_object_created;
        cb.on_object_destroyed = callbacks.on_object_destroyed;
        cb.on_error_occurred = callbacks.on_error_occurred;
        cb.on_object_id_remapped = callbacks.on_object_id_remapped;
    }
    
    // Set up connection parameters
    let connection_params = net::ConnectionParams {
        host: config.host,
        database_name: config.db_name,
        auth_token: if !config.auth_token.is_empty() {
            Some(config.auth_token)
        } else {
            None
        },
    };
    
    // Set up handlers
    let on_connected = {
        let on_connected_ptr = callbacks.on_connected;
        move || {
            invoke_on_connected(on_connected_ptr);
        }
    };
    
    let on_disconnected = {
        let on_disconnected_ptr = callbacks.on_disconnected;
        move |reason: &str| {
            invoke_on_disconnected(on_disconnected_ptr, reason);
        }
    };
    
    let on_error = {
        let on_error_ptr = callbacks.on_error_occurred;
        move |error: &str| {
            invoke_on_error(on_error_ptr, error);
        }
    };
    
    let on_property_updated = {
        let on_property_updated_ptr = callbacks.on_property_updated;
        move |object_id: ObjectId, property_name: &str, value_json: &str| {
            invoke_on_property_updated(on_property_updated_ptr, object_id, property_name, value_json);
        }
    };
    
    let on_object_created = {
        let on_object_created_ptr = callbacks.on_object_created;
        move |object_id: ObjectId, class_name: &str, data_json: &str| {
            invoke_on_object_created(on_object_created_ptr, object_id, class_name, data_json);
        }
    };
    
    let on_object_destroyed = {
        let on_object_destroyed_ptr = callbacks.on_object_destroyed;
        move |object_id: ObjectId| {
            invoke_on_object_destroyed(on_object_destroyed_ptr, object_id);
        }
    };
    
    // Connect to server
    match net::connect(connection_params) {
        Ok(_) => {
            // Set up event handlers
            net::set_event_handlers(
                on_connected,
                on_disconnected,
                on_error,
                on_property_updated,
                on_object_created,
                on_object_destroyed,
            );
            true
        },
        Err(err) => {
            let error_msg = format!("Failed to connect: {}", err);
            invoke_on_error(callbacks.on_error_occurred, &error_msg);
            false
        }
    }
}

/// Disconnect from the SpacetimeDB server
fn disconnect_from_server() -> bool {
    net::disconnect()
}

/// Check if we're connected to the server
fn is_connected() -> bool {
    net::is_connected()
}

/// Set a property on an object
fn set_property(object_id: u64, property_name: &CxxString, value_json: &CxxString, replicate: bool) -> bool {
    let prop_name = property_name.to_string();
    let value_json_str = value_json.to_string();
    
    // Deserialize the value from JSON
    match property::serialization::deserialize_property_value(&value_json_str) {
        Ok(value) => {
            // Set the property
            match crate::set_property(object_id, &prop_name, value, replicate) {
                Ok(_) => true,
                Err(err) => {
                    let error_msg = format!("Failed to set property: {}", err);
                    let error_ptr = CALLBACKS.lock().unwrap().on_error_occurred;
                    invoke_on_error(error_ptr, &error_msg);
                    false
                }
            }
        },
        Err(err) => {
            let error_msg = format!("Failed to parse property value: {}", err);
            let error_ptr = CALLBACKS.lock().unwrap().on_error_occurred;
            invoke_on_error(error_ptr, &error_msg);
            false
        }
    }
}

/// Get a property from an object
fn get_property(object_id: u64, property_name: &CxxString) -> CxxString {
    let prop_name = property_name.to_string();
    
    // Look up the property
    if let Some(value) = property::get_cached_property_value(object_id, &prop_name) {
        // Serialize to JSON
        match property::serialization::serialize_property_value(&value) {
            Ok(json) => CxxString::new(&json),
            Err(_) => CxxString::new(""),
        }
    } else {
        CxxString::new("")
    }
}

/// Create a new object
fn create_object(class_name: &CxxString, params_json: &CxxString) -> u64 {
    let class_name_str = class_name.to_string();
    let params_json_str = params_json.to_string();
    
    // Parse spawn parameters
    match serde_json::from_str::<stdb_shared::object::SpawnParams>(&params_json_str) {
        Ok(params) => {
            // Create object
            match object::create_object(&class_name_str, params) {
                Ok(object_id) => object_id,
                Err(err) => {
                    let error_msg = format!("Failed to create object: {}", err);
                    let error_ptr = CALLBACKS.lock().unwrap().on_error_occurred;
                    invoke_on_error(error_ptr, &error_msg);
                    0
                }
            }
        },
        Err(err) => {
            let error_msg = format!("Failed to parse spawn parameters: {}", err);
            let error_ptr = CALLBACKS.lock().unwrap().on_error_occurred;
            invoke_on_error(error_ptr, &error_msg);
            0
        }
    }
}

/// Destroy an object
fn destroy_object(object_id: u64) -> bool {
    match object::destroy_object(object_id) {
        Ok(_) => true,
        Err(err) => {
            let error_msg = format!("Failed to destroy object: {}", err);
            let error_ptr = CALLBACKS.lock().unwrap().on_error_occurred;
            invoke_on_error(error_ptr, &error_msg);
            false
        }
    }
}

/// Call a function on the server
fn call_server_function(object_id: u64, function_name: &CxxString, args_json: &CxxString) -> bool {
    let function_name_str = function_name.to_string();
    let args_json_str = args_json.to_string();
    
    match crate::call_server_rpc(object_id, &function_name_str, &args_json_str) {
        Ok(_) => true,
        Err(err) => {
            let error_msg = format!("Failed to call server function: {}", err);
            let error_ptr = CALLBACKS.lock().unwrap().on_error_occurred;
            invoke_on_error(error_ptr, &error_msg);
            false
        }
    }
}

/// Register a client function handler
fn register_client_function(function_name: &CxxString, handler_ptr: usize) -> bool {
    let function_name_str = function_name.to_string();
    
    if handler_ptr == 0 {
        let error_msg = "Invalid function handler pointer";
        let error_ptr = CALLBACKS.lock().unwrap().on_error_occurred;
        invoke_on_error(error_ptr, error_msg);
        return false;
    }
    
    // Create a handler that will call the C++ function
    let handler = Box::new(move |object_id: ObjectId, args_json: &str| -> Result<(), String> {
        // Convert function pointer to the correct type
        let func: unsafe extern "C" fn(u64, *const c_char) -> bool = 
            unsafe { std::mem::transmute(handler_ptr) };
        
        // Call the C++ function
        let c_args_json = CString::new(args_json)
            .map_err(|e| format!("Failed to convert args to C string: {}", e))?;
        
        let success = unsafe { func(object_id, c_args_json.as_ptr()) };
        
        if success {
            Ok(())
        } else {
            Err("Function handler returned failure".to_string())
        }
    });
    
    // Register the handler
    crate::register_client_rpc(&function_name_str, handler);
    true
}

/// Get the client ID
fn get_client_id() -> u64 {
    net::get_client_id()
}

/// Get an object's class name
fn get_object_class(object_id: u64) -> CxxString {
    match object::get_object_class(object_id) {
        Some(class_name) => CxxString::new(&class_name),
        None => CxxString::new(""),
    }
} 