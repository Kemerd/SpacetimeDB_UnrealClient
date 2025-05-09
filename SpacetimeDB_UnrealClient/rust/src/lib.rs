// SpacetimeDB Unreal Client - Rust FFI Layer
// This file defines the Foreign Function Interface (FFI) between Rust and C++
// for the SpacetimeDB Unreal Engine plugin. It uses the `cxx` crate to generate
// the necessary bindings.

// Import necessary modules from the `cxx` crate for FFI.
use cxx::{CxxString, CxxVector};
// Import standard library modules.
use std::sync::{Arc, Mutex};
use std::str::FromStr;
use std::collections::HashMap;
use once_cell::sync::Lazy;
use serde_json::{json, Value};

// SpacetimeDB SDK imports
use spacetimedb_sdk::{
    Address, Client, ClientBuilder, Identity, ReducerCallError, 
    subscribe::SubscriptionHandle, ConnectionConfig as SDKConnectionConfig,
    identity::IdentityConfig, table, reducer,
};

// --- Global state for SpacetimeDB client ---

// Actual SpacetimeDB client wrapped in thread-safe containers
static CLIENT: Lazy<Mutex<Option<Client>>> = Lazy::new(|| Mutex::new(None));
static CALLBACKS: Lazy<Mutex<EventCallbacks>> = Lazy::new(|| Mutex::new(EventCallbacks::default()));
static SUBSCRIPTIONS: Lazy<Mutex<HashMap<String, SubscriptionHandle>>> = Lazy::new(|| Mutex::new(HashMap::new()));

// Struct to store C++ callback function pointers
#[derive(Debug, Default)]
struct EventCallbacks {
    on_connected: usize,
    on_disconnected: usize,
    on_identity_received: usize,
    on_event_received: usize,
    on_error_occurred: usize,
}

// --- FFI Bridge Definition ---
// This section uses `cxx::bridge` to define the interface between Rust and C++.
// Functions declared in `extern "Rust"` blocks are implemented in Rust and callable from C++.
// Types declared here can be shared between Rust and C++.
#[cxx::bridge(namespace = "stdb::ffi")]
mod ffi {
    // Configuration for connecting to SpacetimeDB.
    // Passed from C++ to Rust.
    struct ConnectionConfig {
        host: String,
        db_name: String,
        // Optional: credentials if your SpacetimeDB instance requires them.
        // For example, an API key or token.
        auth_token: String, 
    }

    // Struct to pass callback function pointers from C++ to Rust.
    // `usize` is used to represent raw function pointers.
    #[derive(Debug, Default)]
    struct EventCallbackPointers {
        on_connected: usize,        // void (*)()
        on_disconnected: usize,     // void (*)(const char* reason)
        on_identity_received: usize, // void (*)(const char* identity_hex)
        on_event_received: usize,   // void (*)(const char* event_data_json, const char* table_name)
        on_error_occurred: usize,   // void (*)(const char* error_message)
    }

    // Functions exposed from Rust to C++.
    extern "Rust" {
        // Attempts to connect to the SpacetimeDB server.
        // `config`: Connection parameters.
        // `callbacks`: Pointers to C++ functions to be called on various events.
        // Returns `true` if the connection attempt was initiated, `false` otherwise.
        fn connect_to_server(config: ConnectionConfig, callbacks: EventCallbackPointers) -> bool;

        // Disconnects from the SpacetimeDB server.
        // Returns `true` if disconnection was initiated, `false` otherwise.
        fn disconnect_from_server() -> bool;

        // Checks if the client is currently connected.
        // Returns `true` if connected, `false` otherwise.
        fn is_client_connected() -> bool;

        // Sends a message (e.g., a reducer call) to the SpacetimeDB server.
        // `reducer_name`: The name of the reducer to call.
        // `args_json`: A JSON string representing the arguments for the reducer.
        // Returns `true` if the message was successfully queued for sending, `false` otherwise.
        fn call_reducer(reducer_name: &CxxString, args_json: &CxxString) -> bool;
        
        // Subscribes to one or more tables.
        // `table_names`: A vector of table names to subscribe to.
        // Returns `true` if the subscription request was successfully queued, `false` otherwise.
        fn subscribe_to_tables(table_names: &CxxVector<CxxString>) -> bool;

        // Returns the client's identity as a hex string, or empty string if not available.
        fn get_client_identity() -> CxxString;
    }
}

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

fn invoke_on_identity(cb_ptr: usize, identity_hex: &str) {
    if cb_ptr != 0 {
        let func: unsafe extern "C" fn(*const c_char) = unsafe { std::mem::transmute(cb_ptr) };
        let c_identity = CString::new(identity_hex).unwrap_or_else(|_| CString::new("").unwrap());
        unsafe { func(c_identity.as_ptr()) };
    }
}

fn invoke_on_event(cb_ptr: usize, event_data_json: &str, table_name: &str) {
    if cb_ptr != 0 {
        let func: unsafe extern "C" fn(*const c_char, *const c_char) = unsafe { std::mem::transmute(cb_ptr) };
        let c_event_data = CString::new(event_data_json).unwrap_or_else(|_| CString::new("{}").unwrap());
        let c_table_name = CString::new(table_name).unwrap_or_else(|_| CString::new("").unwrap());
        unsafe { func(c_event_data.as_ptr(), c_table_name.as_ptr()) };
    }
}

fn invoke_on_error(cb_ptr: usize, error_message: &str) {
    if cb_ptr != 0 {
        let func: unsafe extern "C" fn(*const c_char) = unsafe { std::mem::transmute(cb_ptr) };
        let c_error_message = CString::new(error_message).unwrap_or_else(|_| CString::new("unknown error").unwrap());
        unsafe { func(c_error_message.as_ptr()) };
    }
}

// --- Implementation of FFI functions ---

// Connects to the SpacetimeDB server using the provided configuration and callbacks.
fn connect_to_server(config: ffi::ConnectionConfig, callbacks: ffi::EventCallbackPointers) -> bool {
    println!(
        "Rust: Attempting to connect to SpacetimeDB at {}:{} with auth token: '{}'",
        config.host, config.db_name, config.auth_token
    );
    
    // Store callback pointers for later use
    {
        let mut event_callbacks = CALLBACKS.lock().unwrap();
        event_callbacks.on_connected = callbacks.on_connected;
        event_callbacks.on_disconnected = callbacks.on_disconnected;
        event_callbacks.on_identity_received = callbacks.on_identity_received;
        event_callbacks.on_event_received = callbacks.on_event_received;
        event_callbacks.on_error_occurred = callbacks.on_error_occurred;
    }
    
    // Check if already connected or connecting
    {
        let client_lock = CLIENT.lock().unwrap();
        if client_lock.is_some() {
            println!("Rust: Already connected or connection attempt in progress.");
            let error_msg = "Already connected or connection in progress";
            let cb_ptr = CALLBACKS.lock().unwrap().on_error_occurred;
            invoke_on_error(cb_ptr, error_msg);
            return false;
        }
    }
    
    // Prepare connection config for SpacetimeDB SDK
    let mut sdk_config = SDKConnectionConfig::new();
    if !config.auth_token.is_empty() {
        sdk_config = sdk_config.with_auth_token(&config.auth_token);
    }

    // Setup URL with host and database name
    let url = format!("{}/{}", config.host, config.db_name);
    println!("Rust: Connecting to URL: {}", url);
    
    // Setup identity config (create new identity or load existing)
    let identity_config = IdentityConfig::generate_ephemeral();
    
    // Create and configure client
    match ClientBuilder::new(&url)
        .with_identity_config(identity_config)
        .with_connection_config(sdk_config)
        .connect() {
        Ok(client) => {
            // Set up connection event handlers
            let on_connect = {
                let callbacks = CALLBACKS.lock().unwrap();
                let on_connected_ptr = callbacks.on_connected;
                let on_identity_ptr = callbacks.on_identity_received;
                
                move || {
                    println!("Rust: Connected to SpacetimeDB successfully.");
                    invoke_on_connected(on_connected_ptr);
                    
                    // Get and notify about identity
                    if let Some(ref client_ref) = *CLIENT.lock().unwrap() {
                        if let Some(identity) = client_ref.get_identity() {
                            let identity_hex = identity.to_hex();
                            println!("Rust: Client identity: {}", identity_hex);
                            invoke_on_identity(on_identity_ptr, &identity_hex);
                        }
                    }
                }
            };
            
            let on_disconnect = {
                let on_disconnected_ptr = CALLBACKS.lock().unwrap().on_disconnected;
                move |reason: &str| {
                    println!("Rust: Disconnected from SpacetimeDB: {}", reason);
                    invoke_on_disconnected(on_disconnected_ptr, reason);
                }
            };
            
            let on_error = {
                let on_error_ptr = CALLBACKS.lock().unwrap().on_error_occurred;
                move |error: &str| {
                    println!("Rust: Error from SpacetimeDB: {}", error);
                    invoke_on_error(on_error_ptr, error);
                }
            };
            
            // Set up event handling
            let on_event = {
                let on_event_ptr = CALLBACKS.lock().unwrap().on_event_received;
                move |table_name: &str, event_json: &str| {
                    println!("Rust: Received event for table '{}': {}", table_name, event_json);
                    invoke_on_event(on_event_ptr, event_json, table_name);
                }
            };
            
            // Configure client with callbacks
            let mut client_with_callbacks = client
                .with_on_connect(on_connect)
                .with_on_disconnect(on_disconnect);
                
            // Store client in global state
            let mut client_lock = CLIENT.lock().unwrap();
            *client_lock = Some(client_with_callbacks);
            
            println!("Rust: Connection initiated to SpacetimeDB.");
            true
        },
        Err(e) => {
            let error_message = format!("Failed to connect to SpacetimeDB: {}", e);
            println!("Rust: {}", error_message);
            let on_error_ptr = CALLBACKS.lock().unwrap().on_error_occurred;
            invoke_on_error(on_error_ptr, &error_message);
            false
        }
    }
}

// Disconnects from the SpacetimeDB server.
fn disconnect_from_server() -> bool {
    println!("Rust: Attempting to disconnect from SpacetimeDB.");
    
    let mut client_lock = CLIENT.lock().unwrap();
    if let Some(client) = client_lock.take() {
        // Clean up subscriptions
        {
            let mut subs_lock = SUBSCRIPTIONS.lock().unwrap();
            subs_lock.clear();
        }
        
        // Disconnect the client
        // The SDK will automatically call the on_disconnect callback
        drop(client);
        
        println!("Rust: Disconnected from SpacetimeDB.");
        true
    } else {
        println!("Rust: No active connection to disconnect.");
        false
    }
}

// Checks if the client is currently connected.
fn is_client_connected() -> bool {
    let client_lock = CLIENT.lock().unwrap();
    match *client_lock {
        Some(ref client) => client.is_connected(),
        None => false
    }
}

// Sends a message (e.g., a reducer call) to the SpacetimeDB server.
fn call_reducer(reducer_name: &CxxString, args_json: &CxxString) -> bool {
    let name_str = reducer_name.to_string_lossy();
    let args_str = args_json.to_string_lossy();
    println!(
        "Rust: Attempting to call reducer '{}' with args: {}",
        name_str, args_str
    );
    
    // Ensure client is connected
    if !is_client_connected() {
        let error_msg = "Cannot call reducer: Client not connected";
        println!("Rust: {}", error_msg);
        let on_error_ptr = CALLBACKS.lock().unwrap().on_error_occurred;
        invoke_on_error(on_error_ptr, error_msg);
        return false;
    }
    
    // Parse args_json into a Value for the reducer call
    let args_value = match serde_json::from_str::<Value>(&args_str) {
        Ok(value) => value,
        Err(e) => {
            let error_msg = format!("Failed to parse reducer arguments JSON: {}", e);
            println!("Rust: {}", error_msg);
            let on_error_ptr = CALLBACKS.lock().unwrap().on_error_occurred;
            invoke_on_error(on_error_ptr, &error_msg);
            return false;
        }
    };
    
    // Call the reducer with the provided name and arguments
    let client_lock = CLIENT.lock().unwrap();
    if let Some(ref client) = *client_lock {
        match client.call_reducer(&name_str, &args_value) {
            Ok(_) => {
                println!("Rust: Successfully called reducer '{}'", name_str);
                true
            },
            Err(e) => {
                let error_msg = format!("Failed to call reducer '{}': {}", name_str, e);
                println!("Rust: {}", error_msg);
                let on_error_ptr = CALLBACKS.lock().unwrap().on_error_occurred;
                invoke_on_error(on_error_ptr, &error_msg);
                false
            }
        }
    } else {
        // This shouldn't happen as we already checked is_client_connected()
        let error_msg = "Client disappeared during reducer call";
        println!("Rust: {}", error_msg);
        let on_error_ptr = CALLBACKS.lock().unwrap().on_error_occurred;
        invoke_on_error(on_error_ptr, error_msg);
        false
    }
}

// Subscribes to one or more tables.
fn subscribe_to_tables(table_names: &CxxVector<CxxString>) -> bool {
    let tables: Vec<String> = table_names.iter().map(|s| s.to_string_lossy().into_owned()).collect();
    println!("Rust: Attempting to subscribe to tables: {:?}", tables);
    
    // Ensure client is connected
    if !is_client_connected() {
        let error_msg = "Cannot subscribe: Client not connected";
        println!("Rust: {}", error_msg);
        let on_error_ptr = CALLBACKS.lock().unwrap().on_error_occurred;
        invoke_on_error(on_error_ptr, error_msg);
        return false;
    }
    
    let client_lock = CLIENT.lock().unwrap();
    if let Some(ref client) = *client_lock {
        let mut success = true;
        let mut subs_lock = SUBSCRIPTIONS.lock().unwrap();
        
        // Subscribe to each table
        for table_name in tables {
            // Skip if already subscribed
            if subs_lock.contains_key(&table_name) {
                println!("Rust: Already subscribed to table '{}'", table_name);
                continue;
            }
            
            // Create subscription handler for this table
            let on_event_ptr = CALLBACKS.lock().unwrap().on_event_received;
            let table_name_clone = table_name.clone();
            let event_handler = move |json_data: String| {
                println!("Rust: Received event for table '{}': {}", table_name_clone, json_data);
                invoke_on_event(on_event_ptr, &json_data, &table_name_clone);
            };
            
            // Subscribe to the table with the created handler
            match client.subscribe_to_table(&table_name, event_handler) {
                Ok(handle) => {
                    println!("Rust: Successfully subscribed to table '{}'", table_name);
                    subs_lock.insert(table_name, handle);
                },
                Err(e) => {
                    let error_msg = format!("Failed to subscribe to table '{}': {}", table_name, e);
                    println!("Rust: {}", error_msg);
                    let on_error_ptr = CALLBACKS.lock().unwrap().on_error_occurred;
                    invoke_on_error(on_error_ptr, &error_msg);
                    success = false;
                }
            }
        }
        
        success
    } else {
        // This shouldn't happen as we already checked is_client_connected()
        let error_msg = "Client disappeared during subscription request";
        println!("Rust: {}", error_msg);
        let on_error_ptr = CALLBACKS.lock().unwrap().on_error_occurred;
        invoke_on_error(on_error_ptr, error_msg);
        false
    }
}

fn get_client_identity() -> CxxString {
    let client_lock = CLIENT.lock().unwrap();
    if let Some(ref client) = *client_lock {
        if let Some(identity) = client.get_identity() {
            let identity_hex = identity.to_hex();
            println!("Rust: Retrieving client identity: {}", identity_hex);
            return CxxString::new(&identity_hex);
        }
    }
    
    println!("Rust: No identity available");
    CxxString::new("")
}

// For standalone testing if compiled as a binary
fn main() {
    println!("Rust FFI library for SpacetimeDB Unreal Client. This main function is not used when compiled as a library.");
}

// --- Unit Tests ---
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_flow() {
        // Set up a mock SpacetimeDB server would be ideal here
        // but for now we'll just test the API without connecting
        
        let config = ffi::ConnectionConfig {
            host: "localhost".to_string(),
            db_name: "testdb".to_string(),
            auth_token: "testtoken".to_string(),
        };
        
        let callbacks = ffi::EventCallbackPointers::default();
        
        // Note: These tests won't actually connect to a server,
        // they're just checking the API behaves as expected.
        // For real tests, we'd need a mock server or integration tests.
        
        // Testing connection would go here
        // Testing subscriptions would go here
        // Testing reducer calls would go here
    }
} 