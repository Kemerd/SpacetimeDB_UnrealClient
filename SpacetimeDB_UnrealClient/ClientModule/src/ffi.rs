//! # FFI Layer
//!
//! Foreign Function Interface layer that bridges between Rust and C++ code.
//! This provides the integration between our Rust client module and Unreal Engine.

use cxx::{CxxString, CxxVector, UniquePtr};
use std::sync::{Arc, Mutex};
use std::str::FromStr;
use std::collections::HashMap;
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};
use stdb_shared::object::ObjectId;
use stdb_shared::property::{PropertyType, PropertyValue, ReplicationCondition};
use stdb_shared::types::*;
use crate::property;
use crate::object;
use crate::net;
use crate::rpc;
use crate::prediction::{get_prediction_system, PredictedTransformUpdate, SequenceNumber};
use stdb_shared::types::Transform;
use log;
use crate::{class, prediction};
use std::ffi::{c_char, CString};

// Define the VelocityData struct since it doesn't seem to exist in the shared types
// This matches the usage in the prediction module
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VelocityData {
    pub linear: [f32; 3],
    pub angular: [f32; 3],
}

// --- Global state for FFI callbacks ---

/// Callback function pointers for C++ integration
#[derive(Debug, Default)]
struct EventCallbacks {
    pub on_connected: usize,
    pub on_disconnected: usize,
    pub on_property_updated: usize,
    pub on_object_created: usize,
    pub on_object_destroyed: usize,
    pub on_error_occurred: usize,
    pub on_object_id_remapped: usize,
}

/// Global storage for callback function pointers
pub static CALLBACKS: Lazy<Mutex<EventCallbacks>> = Lazy::new(|| Mutex::new(EventCallbacks::default()));

// --- Helper functions for callback invocation ---
// These functions safely convert `usize` back to function pointers and call them.
// They are marked `unsafe` because they dereference raw pointers.

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

/// Invoke the callback for when an object is destroyed
pub fn invoke_on_object_destroyed(cb_ptr: usize, object_id: ObjectId) {
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
#[cxx::bridge]
mod ffi {
    // External C++ types
    unsafe extern "C++" {
        include!("UnrealReplication.h");
        include!("bridge.h");
        
        // Use function from global namespace
        fn make_unique_string(s: &str) -> UniquePtr<CxxString>;
    }
    
    // Rust types exposed to C++
    #[derive(Debug)]
    pub enum ConnectionState {
        Disconnected = 0,
        Connecting = 1,
        Connected = 2,
    }
    
    #[derive(Debug)]
    pub enum ReplicationCondition {
        Never = 0,
        OnChange = 1,
        Initial = 2,
        Always = 3,
    }
    
    // Connection configuration
    pub struct ConnectionConfig {
        pub host: String,
        pub db_name: String,
        pub auth_token: String,
    }
    
    // Event callback function pointers
    pub struct EventCallbackPointers {
        pub on_connected: usize,
        pub on_disconnected: usize,
        pub on_property_updated: usize,
        pub on_object_created: usize,
        pub on_object_destroyed: usize,
        pub on_error_occurred: usize,
        pub on_object_id_remapped: usize,
    }
    
    // Property system functions
    extern "Rust" {
        fn create_class(class_name: &CxxString, parent_class_name: &CxxString) -> bool;
        
        fn add_property(
            class_name: &CxxString,
            property_name: &CxxString,
            type_name: &CxxString,
            replicated: bool,
            replication_condition: ReplicationCondition,
            readonly: bool,
            flags: u32,
        ) -> bool;
        
        fn get_property_definition(class_name: &CxxString, property_name: &CxxString) -> UniquePtr<CxxString>;
        
        fn get_property_names_for_class(class_name: &CxxString) -> UniquePtr<CxxString>;
        
        fn get_registered_class_names() -> UniquePtr<CxxString>;
        
        fn export_property_definitions_as_json() -> UniquePtr<CxxString>;
        
        fn import_property_definitions_from_json(json: &CxxString) -> bool;
    }
    
    // Object system functions
    extern "Rust" {
        fn register_object(class_name: &CxxString, params: &CxxString) -> u64;
        
        fn get_object_class(object_id: u64) -> UniquePtr<CxxString>;
        
        fn set_property(object_id: u64, property_name: &CxxString, value_json: &CxxString, replicate: bool) -> bool;
        
        fn get_property(object_id: u64, property_name: &CxxString) -> UniquePtr<CxxString>;
        
        fn dispatch_unreliable_rpc(object_id: u64, function_name: &CxxString, params: &CxxString) -> bool;
    }
    
    // Connection functions
    extern "Rust" {
        fn connect_to_server(config: ConnectionConfig, callbacks: EventCallbackPointers) -> bool;
        fn disconnect_from_server() -> bool;
        fn is_connected() -> bool;
    }
}

// --- FFI Implementation ---

/// Connect to the SpacetimeDB server
fn connect_to_server(config: ffi::ConnectionConfig, callbacks: ffi::EventCallbackPointers) -> bool {
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
    let connection_params = stdb_shared::connection::ConnectionParams {
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
    let property_name_str = property_name.to_str().unwrap_or("");
    let value_str = value_json.to_str().unwrap_or("null");
    
    // Try to deserialize the value JSON
    match serde_json::from_str::<serde_json::Value>(value_str) {
        Ok(value) => {
            // Set the property with the given value
            match object::set_object_property_with_replication(object_id, property_name_str, &value, replicate) {
                Ok(_) => true,
                Err(err) => {
                    log::error!("Failed to set property {}: {}", property_name_str, err);
                    false
                }
            }
        },
        Err(err) => {
            log::error!("Failed to parse value JSON for property {}: {}", property_name_str, err);
            false
        }
    }
}

/// Get a property from an object
fn get_property(object_id: u64, property_name: &CxxString) -> UniquePtr<CxxString> {
    let property_name_str = property_name.to_string();
    
    // Try to get the property
    match object::get_property(object_id, &property_name_str) {
        Ok(Some(value)) => {
            match property::serialize_property_value(&value) {
                Ok(json) => {
                    // Create a new CxxString with the JSON value
                    create_cxx_string(&json)
                },
                Err(e) => {
                    // Create a new CxxString with the error message
                    create_cxx_string(&format!("Error serializing property: {}", e))
                }
            }
        },
        Ok(None) => {
            // Return an empty string for properties that don't exist
            create_cxx_string("")
        },
        Err(e) => {
            // Create a new CxxString with the error message
            create_cxx_string(&format!("Error getting property: {}", e))
        }
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
fn get_object_class(object_id: u64) -> UniquePtr<CxxString> {
    // Try to get the object's class
    match object::get_object_class(object_id) {
        Some(class_name) => {
            // Create a new CxxString with the class name
            create_cxx_string(&class_name)
        },
        None => {
            // Return an empty string if the object doesn't exist
            create_cxx_string("")
        }
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

/// Get all property names for a class as a JSON array
fn get_property_names_for_class(class_name: &CxxString) -> UniquePtr<CxxString> {
    let class_name_str = class_name.to_string();
    
    // Check if property definitions exist for this class
    if !property::has_property_definitions_for_class(&class_name_str) {
        // Return empty array if no property definitions exist
        return create_cxx_string("[]");
    }
    
    // Get the property names as JSON
    match property::get_property_names_for_class_as_json(&class_name_str) {
        Ok(json) => {
            create_cxx_string(&json)
        },
        Err(_) => {
            // Return empty array on error
            create_cxx_string("[]")
        }
    }
}

/// Get all registered class names as a JSON array
fn get_registered_class_names() -> UniquePtr<CxxString> {
    // Get the registered class names as JSON
    match property::get_registered_class_names_as_json() {
        Ok(json) => {
            create_cxx_string(&json)
        },
        Err(_) => {
            // Return empty array on error
            create_cxx_string("[]")
        }
    }
}

/// Import property definitions from a JSON string
fn import_property_definitions_from_json(json: &CxxString) -> bool {
    let json_str = json.to_str().unwrap_or("{}");
    
    // Try to import the property definitions from JSON
    match property::import_property_definitions_from_json(json_str) {
        Ok(count) => {
            log::info!("Imported {} property definitions", count);
            true
        },
        Err(err) => {
            log::error!("Failed to import property definitions: {}", err);
            false
        }
    }
}

/// Export all property definitions as a JSON string
fn export_property_definitions_as_json() -> UniquePtr<CxxString> {
    // Get the property definitions as JSON
    match property::export_property_definitions_as_json() {
        Ok(json) => {
            create_cxx_string(&json)
        },
        Err(_) => {
            // Return empty object on error
            create_cxx_string("{}")
        }
    }
}

/// Get a property definition as a JSON string
fn get_property_definition(class_name: &CxxString, property_name: &CxxString) -> UniquePtr<CxxString> {
    let class_name_str = class_name.to_string();
    let property_name_str = property_name.to_string();
    
    // Get the property definition as JSON
    match property::get_property_definition_as_json(&class_name_str, &property_name_str) {
        Ok(json) => {
            create_cxx_string(&json)
        },
        Err(_) => {
            // Return empty object if property doesn't exist
            create_cxx_string("{}")
        }
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
fn get_components(actor_id: u64) -> UniquePtr<CxxString> {
    // Try to get the components
    match object::get_components(actor_id) {
        Ok(components) => {
            if let Ok(json) = serde_json::to_string(&components) {
                // Create a new CxxString with the JSON components list
                create_cxx_string(&json)
            } else {
                // Return empty array on error
                create_cxx_string("[]")
            }
        },
        Err(_) => {
            // Return empty array if no components found
            create_cxx_string("[]")
        }
    }
}

/// Get a component by class name, returns 0 if not found
fn get_component_by_class(actor_id: u64, class_name: &CxxString) -> u64 {
    let class_name_str = class_name.to_string();
    match object::get_component_by_class(actor_id, &class_name_str) {
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
    let component_class_str = component_class.to_string();
    match object::create_and_attach_component(actor_id, &component_class_str) {
        Ok(component_id) => component_id,
        Err(e) => {
            eprintln!("Failed to create and attach component: {}", e);
            0
        }
    }
}

/// Get a property from a component
fn get_component_property(actor_id: u64, component_class: &CxxString, property_name: &CxxString) -> UniquePtr<CxxString> {
    let component_class_str = component_class.to_string();
    let property_name_str = property_name.to_string();
    
    // Try to get the property from the component
    match object::get_component_property(actor_id, &component_class_str, &property_name_str) {
        Ok(Some(value)) => {
            match property::serialize_property_value(&value) {
                Ok(json) => {
                    // Create a new CxxString with the JSON value
                    create_cxx_string(&json)
                },
                Err(e) => {
                    // Create a new CxxString with the error message
                    create_cxx_string(&format!("Error serializing component property: {}", e))
                }
            }
        },
        Ok(None) => {
            // Return an empty string for properties that don't exist
            create_cxx_string("")
        },
        Err(e) => {
            // Create a new CxxString with the error message
            create_cxx_string(&format!("Error getting component property: {}", e))
        }
    }
}

/// Set a property on a component
fn set_component_property(actor_id: u64, component_class: &CxxString, property_name: &CxxString, value_json: &CxxString) -> bool {
    let component_class_str = component_class.to_string();
    let property_name_str = property_name.to_string();
    let value_json_str = value_json.to_string();
    
    // Parse the JSON value into a PropertyValue
    let value_result = property::serialization::deserialize_property_value(&value_json_str);
    if let Err(e) = value_result {
        eprintln!("Failed to deserialize property value: {}", e);
        return false;
    }
    
    // Update the component property
    match object::set_component_property(actor_id, &component_class_str, &property_name_str, value_result.unwrap()) {
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
    let transform = Transform {
        location: Vector3 { x: location_x, y: location_y, z: location_z },
        rotation: Quat { x: rotation_x, y: rotation_y, z: rotation_z, w: rotation_w },
        scale: Vector3 { x: scale_x, y: scale_y, z: scale_z },
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

fn create_class(class_name: &CxxString, parent_class_name: &CxxString) -> bool {
    let class_name_str = class_name.to_str().unwrap_or("");
    let parent_class_name_str = parent_class_name.to_str().unwrap_or("");
    
    // Create the class with the given parent class name
    class::create_class(class_name_str, parent_class_name_str)
}

fn add_property(
    class_name: &CxxString,
    property_name: &CxxString,
    type_name: &CxxString,
    replicated: bool,
    replication_condition: ffi::ReplicationCondition,
    readonly: bool,
    flags: u32,
) -> bool {
    let class_name_str = class_name.to_str().unwrap_or("");
    let property_name_str = property_name.to_str().unwrap_or("");
    let type_name_str = type_name.to_str().unwrap_or("");
    
    // Convert from FFI ReplicationCondition to internal ReplicationCondition
    let repl_condition = match replication_condition {
        ffi::ReplicationCondition::Never => stdb_shared::property::ReplicationCondition::Initial, // Changed to Initial as Never doesn't exist
        ffi::ReplicationCondition::OnChange => stdb_shared::property::ReplicationCondition::OnChange,
        ffi::ReplicationCondition::Initial => stdb_shared::property::ReplicationCondition::Initial,
        ffi::ReplicationCondition::Always => stdb_shared::property::ReplicationCondition::Always,
        _ => stdb_shared::property::ReplicationCondition::OnChange, // Default for any other values
    };
    
    property::add_property(
        class_name_str,
        property_name_str,
        type_name_str,
        replicated,
        repl_condition,
        readonly,
        flags
    )
}

fn register_object(class_name: &CxxString, params: &CxxString) -> u64 {
    let class_name_str = class_name.to_str().unwrap_or("");
    let params_str = params.to_str().unwrap_or("{}");
    
    // Try to deserialize the parameter JSON
    match serde_json::from_str::<serde_json::Value>(params_str) {
        Ok(params_value) => {
            // Register the object with the provided class name and parameters
            match object::register_object(class_name_str, &params_value) {
                Ok(object_id) => object_id,
                Err(err) => {
                    log::error!("Failed to register object: {}", err);
                    0
                }
            }
        },
        Err(err) => {
            log::error!("Failed to parse parameters JSON: {}", err);
            0
        }
    }
}

fn dispatch_unreliable_rpc(object_id: u64, function_name: &CxxString, params: &CxxString) -> bool {
    let function_name_str = function_name.to_str().unwrap_or("");
    let params_str = params.to_str().unwrap_or("{}");
    
    // Deserialize the parameters
    match serde_json::from_str::<serde_json::Value>(params_str) {
        Ok(params_value) => {
            // Dispatch the RPC
            match object::dispatch_unreliable_rpc(object_id, function_name_str, &params_value) {
                Ok(_) => true,
                Err(err) => {
                    log::error!("Failed to dispatch unreliable RPC {}: {}", function_name_str, err);
                    false
                }
            }
        },
        Err(err) => {
            log::error!("Failed to parse RPC parameters for {}: {}", function_name_str, err);
            false
        }
    }
}

// For each UniquePtr::new() call, we need to add a CxxString parameter
// Here's a helper function to create a new CxxString in a UniquePtr
fn create_cxx_string(s: &str) -> UniquePtr<CxxString> {
    // Use the global namespace function as it's more likely to be found by the linker
    ffi::make_unique_string(s)
} 