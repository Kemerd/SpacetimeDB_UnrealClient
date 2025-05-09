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
use crate::prediction::{get_prediction_system, PredictedTransformUpdate, SequenceNumber};
use crate::object::{TransformData, VelocityData};

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
        
        // Component management
        fn add_component(actor_id: u64, component_id: u64) -> bool;
        fn remove_component(actor_id: u64, component_id: u64) -> bool;
        fn get_components(actor_id: u64) -> CxxString;
        fn get_component_by_class(actor_id: u64, class_name: &CxxString) -> u64;
        fn is_component(object_id: u64) -> bool;
        fn get_component_owner(component_id: u64) -> u64;
        fn create_and_attach_component(actor_id: u64, component_class: &CxxString) -> u64;
        fn get_component_property(actor_id: u64, component_class: &CxxString, property_name: &CxxString) -> CxxString;
        fn set_component_property(actor_id: u64, component_class: &CxxString, property_name: &CxxString, value_json: &CxxString) -> bool;
        
        // RPC (Remote Procedure Calls)
        fn call_server_function(object_id: u64, function_name: &CxxString, args_json: &CxxString) -> bool;
        fn register_client_function(function_name: &CxxString, handler_ptr: usize) -> bool;
        
        // Utility functions
        fn get_client_id() -> u64;
        fn get_object_class(object_id: u64) -> CxxString;
        
        // Property definition functions
        fn get_property_definition_count() -> usize;
        fn has_property_definitions_for_class(class_name: &CxxString) -> bool;
        fn get_property_names_for_class(class_name: &CxxString) -> CxxString;
        fn get_registered_class_names() -> CxxString;
        fn import_property_definitions_from_json(json_str: &CxxString) -> bool;
        fn export_property_definitions_as_json() -> CxxString;
        fn get_property_definition(class_name: &CxxString, property_name: &CxxString) -> CxxString;
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

/// Get the total number of property definitions
fn get_property_definition_count() -> usize {
    crate::get_property_definition_count()
}

/// Check if property definitions are available for a specific class
fn has_property_definitions_for_class(class_name: &CxxString) -> bool {
    let class_name_str = class_name.to_string();
    crate::has_property_definitions_for_class(&class_name_str)
}

/// Get property names for a specific class as a JSON array
fn get_property_names_for_class(class_name: &CxxString) -> CxxString {
    let class_name_str = class_name.to_string();
    let prop_names = crate::get_property_names_for_class(&class_name_str);
    
    // Convert to JSON array
    match serde_json::to_string(&prop_names) {
        Ok(json) => CxxString::new(&json),
        Err(_) => CxxString::new("[]"),
    }
}

/// Get all registered class names as a JSON array
fn get_registered_class_names() -> CxxString {
    let class_names = crate::get_registered_class_names();
    
    // Convert to JSON array
    match serde_json::to_string(&class_names) {
        Ok(json) => CxxString::new(&json),
        Err(_) => CxxString::new("[]"),
    }
}

/// Import property definitions from a JSON string
fn import_property_definitions_from_json(json_str: &CxxString) -> bool {
    let json_str_rust = json_str.to_string();
    
    match crate::import_property_definitions_from_json(&json_str_rust) {
        Ok(_) => true,
        Err(err) => {
            let error_msg = format!("Failed to import property definitions: {}", err);
            let error_ptr = CALLBACKS.lock().unwrap().on_error_occurred;
            invoke_on_error(error_ptr, &error_msg);
            false
        }
    }
}

/// Export all property definitions as a JSON string
fn export_property_definitions_as_json() -> CxxString {
    match crate::export_property_definitions_as_json() {
        Ok(json) => CxxString::new(&json),
        Err(_) => CxxString::new("{}"),
    }
}

/// Get a property definition as a JSON object
fn get_property_definition(class_name: &CxxString, property_name: &CxxString) -> CxxString {
    let class_name_str = class_name.to_string();
    let property_name_str = property_name.to_string();
    
    match crate::get_property_definition(&class_name_str, &property_name_str) {
        Some(def) => {
            // Convert PropertyDefinition to a JSON object
            let json = serde_json::json!({
                "name": def.name,
                "type": format!("{:?}", def.property_type),
                "replicated": def.replicated,
                "replication_condition": format!("{:?}", def.replication_condition),
                "readonly": def.readonly,
                "flags": def.flags
            });
            
            match serde_json::to_string(&json) {
                Ok(json_str) => CxxString::new(&json_str),
                Err(_) => CxxString::new("{}"),
            }
        },
        None => CxxString::new("{}"),
    }
}

/// Add a component to an actor
fn add_component(actor_id: u64, component_id: u64) -> bool {
    match object::add_component(actor_id, component_id) {
        Ok(_) => true,
        Err(e) => {
            eprintln!("Failed to add component {}: {}", component_id, e);
            false
        }
    }
}

/// Remove a component from an actor
fn remove_component(actor_id: u64, component_id: u64) -> bool {
    match object::remove_component(actor_id, component_id) {
        Ok(_) => true,
        Err(e) => {
            eprintln!("Failed to remove component {}: {}", component_id, e);
            false
        }
    }
}

/// Get all components attached to an actor as a JSON array of object IDs
fn get_components(actor_id: u64) -> CxxString {
    match object::get_components(actor_id) {
        Ok(components) => {
            match serde_json::to_string(&components) {
                Ok(json) => json.into(),
                Err(_) => "[]".into(), 
            }
        },
        Err(_) => "[]".into(),
    }
}

/// Get a component by class name, returns 0 if not found
fn get_component_by_class(actor_id: u64, class_name: &CxxString) -> u64 {
    match object::get_component_by_class(actor_id, class_name) {
        Ok(component_opt) => {
            match component_opt {
                Some(component) => component.id,
                None => 0,
            }
        },
        Err(_) => 0,
    }
}

/// Check if an object is a component
fn is_component(object_id: u64) -> bool {
    object::is_component(object_id)
}

/// Get the owner of a component, returns 0 if not a component or no owner
fn get_component_owner(component_id: u64) -> u64 {
    object::get_component_owner(component_id).unwrap_or(0)
}

/// Create a new component and attach it to an actor
fn create_and_attach_component(actor_id: u64, component_class: &CxxString) -> u64 {
    match object::create_and_attach_component(actor_id, component_class) {
        Ok(component_id) => component_id,
        Err(e) => {
            eprintln!("Failed to create and attach component: {}", e);
            0
        }
    }
}

/// Get a property from a component
fn get_component_property(actor_id: u64, component_class: &CxxString, property_name: &CxxString) -> CxxString {
    match object::get_component_property(actor_id, component_class, property_name) {
        Ok(value_opt) => {
            match value_opt {
                Some(value) => {
                    // Serialize the property value to JSON
                    match property::serialization::serialize_property_value(&value) {
                        Ok(json) => json.into(),
                        Err(_) => "null".into(),
                    }
                },
                None => "null".into(),
            }
        },
        Err(_) => "null".into(),
    }
}

/// Set a property on a component
fn set_component_property(actor_id: u64, component_class: &CxxString, property_name: &CxxString, value_json: &CxxString) -> bool {
    // Parse the JSON value into a PropertyValue
    let value_result = property::serialization::deserialize_property_value(value_json);
    if let Err(e) = value_result {
        eprintln!("Failed to deserialize property value: {}", e);
        return false;
    }
    
    // Update the component property
    match object::set_component_property(actor_id, component_class, property_name, value_result.unwrap()) {
        Ok(_) => true,
        Err(e) => {
            eprintln!("Failed to set component property: {}", e);
            false
        }
    }
}

/// Register an object for client-side prediction
#[no_mangle]
pub extern "C" fn register_prediction_object(object_id: u64) -> bool {
    if let Some(pred_system) = get_prediction_system() {
        pred_system.register_object(object_id);
        true
    } else {
        false
    }
}

/// Unregister an object from client-side prediction
#[no_mangle]
pub extern "C" fn unregister_prediction_object(object_id: u64) -> bool {
    if let Some(pred_system) = get_prediction_system() {
        pred_system.unregister_object(object_id);
        true
    } else {
        false
    }
}

/// Get the next sequence number for an object
#[no_mangle]
pub extern "C" fn get_next_prediction_sequence(object_id: u64) -> u32 {
    if let Some(pred_system) = get_prediction_system() {
        pred_system.get_next_sequence(object_id).unwrap_or(0)
    } else {
        0
    }
}

/// Send a predicted transform update
#[no_mangle]
pub extern "C" fn send_predicted_transform(
    object_id: u64,
    sequence: u32,
    location_x: f32,
    location_y: f32,
    location_z: f32,
    rotation_x: f32,
    rotation_y: f32,
    rotation_z: f32,
    rotation_w: f32,
    scale_x: f32,
    scale_y: f32,
    scale_z: f32,
    velocity_x: f32,
    velocity_y: f32,
    velocity_z: f32,
    has_velocity: bool
) -> bool {
    // Create transform data
    let transform = TransformData {
        location: [location_x, location_y, location_z],
        rotation: [rotation_x, rotation_y, rotation_z, rotation_w],
        scale: [scale_x, scale_y, scale_z],
    };
    
    // Create velocity data if provided
    let velocity = if has_velocity {
        Some(VelocityData {
            linear: [velocity_x, velocity_y, velocity_z],
            angular: [0.0, 0.0, 0.0], // We currently don't use angular velocity in prediction
        })
    } else {
        None
    };
    
    // Create the predicted update
    let update = PredictedTransformUpdate {
        object_id,
        sequence,
        transform,
        velocity,
    };
    
    // TODO: Send this update to the server via network
    // For now we'll just acknowledge it locally for testing
    if let Some(pred_system) = get_prediction_system() {
        pred_system.process_ack(object_id, sequence);
        true
    } else {
        false
    }
}

/// Get the last acknowledged sequence number for an object
#[no_mangle]
pub extern "C" fn get_last_acked_sequence(object_id: u64) -> u32 {
    if let Some(pred_system) = get_prediction_system() {
        pred_system.get_last_acked_sequence(object_id).unwrap_or(0)
    } else {
        0
    }
} 