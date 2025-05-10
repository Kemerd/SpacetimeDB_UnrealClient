//! # Network Module
//!
//! Handles communication with the SpacetimeDB server, including connection management
//! and message passing.

use stdb_shared::object::ObjectId;
use stdb_shared::connection::{ConnectionState, ConnectionParams, ClientConnection, DisconnectReason as SharedDisconnectReason};
use spacetimedb_sdk::{
    Address, identity::Identity, 
    reducer::Status as ReducerStatus,
};
use spacetimedb_sdk::client as sdk_client;
use spacetimedb_sdk::messages::TableUpdate;
use spacetimedb_sdk::messages::TableOp;
use spacetimedb_sdk::client::DisconnectReason;
use spacetimedb_sdk::value::Value;

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::future::Future;
use once_cell::sync::Lazy;
use log::{info, debug, error, warn, trace};
use serde_json;

// Global client state
static CLIENT: Lazy<Mutex<Option<sdk_client::Client>>> = Lazy::new(|| Mutex::new(None));
static CONNECTION_STATE: Lazy<Mutex<ConnectionState>> = Lazy::new(|| Mutex::new(ConnectionState::Disconnected));
static CLIENT_ID: Lazy<Mutex<u64>> = Lazy::new(|| Mutex::new(0));
static SUBSCRIPTION: Lazy<Mutex<Option<sdk_client::SubscriptionHandle>>> = Lazy::new(|| Mutex::new(None));

// Callback handlers
type OnConnectedFn = Box<dyn Fn() + Send + 'static>;
type OnDisconnectedFn = Box<dyn Fn(&str) + Send + 'static>;
type OnErrorFn = Box<dyn Fn(&str) + Send + 'static>;
type OnPropertyUpdatedFn = Box<dyn Fn(ObjectId, &str, &str) + Send + 'static>;
type OnObjectCreatedFn = Box<dyn Fn(ObjectId, &str, &str) + Send + 'static>;
type OnObjectDestroyedFn = Box<dyn Fn(ObjectId) + Send + 'static>;

// Event handlers
static ON_CONNECTED: Lazy<Mutex<Option<OnConnectedFn>>> = Lazy::new(|| Mutex::new(None));
static ON_DISCONNECTED: Lazy<Mutex<Option<OnDisconnectedFn>>> = Lazy::new(|| Mutex::new(None));
static ON_ERROR: Lazy<Mutex<Option<OnErrorFn>>> = Lazy::new(|| Mutex::new(None));
static ON_PROPERTY_UPDATED: Lazy<Mutex<Option<OnPropertyUpdatedFn>>> = Lazy::new(|| Mutex::new(None));
static ON_OBJECT_CREATED: Lazy<Mutex<Option<OnObjectCreatedFn>>> = Lazy::new(|| Mutex::new(None));
static ON_OBJECT_DESTROYED: Lazy<Mutex<Option<OnObjectDestroyedFn>>> = Lazy::new(|| Mutex::new(None));

// Add interface for table update handlers
pub struct TableHandler {
    pub table_name: String,
    pub handler: Box<dyn Fn(&TableUpdate) -> Result<(), String> + Send + 'static>,
}

// Registry for table update handlers
static TABLE_HANDLERS: Lazy<Mutex<Vec<TableHandler>>> = 
    Lazy::new(|| Mutex::new(Vec::new()));

/// Register a handler for table updates
pub fn register_table_handler(
    table_name: &str, 
    handler: impl Fn(&TableUpdate) -> Result<(), String> + Send + 'static
) {
    let mut handlers = TABLE_HANDLERS.lock().unwrap();
    handlers.push(TableHandler {
        table_name: table_name.to_string(),
        handler: Box::new(handler),
    });
    debug!("Registered handler for table: {}", table_name);
}

/// Initialize default table handlers
fn init_default_table_handlers() {
    // Register handlers for core tables
    register_table_handler("ObjectInstance", |update| {
        handle_object_instance_update(update)
    });
    
    register_table_handler("ObjectProperty", |update| {
        handle_object_property_update(update)
    });

    register_table_handler("ObjectTransform", |update| {
        handle_object_transform_update(update)
    });
    
    // Add handler for PropertyDefinitionTable
    register_table_handler("PropertyDefinitionTable", |update| {
        handle_property_definition_update(update)
    });
    
    info!("Default table handlers initialized");
}

/// Connect to the SpacetimeDB server
pub fn connect(params: ConnectionParams) -> Result<(), String> {
    // Already connected or connecting
    {
        let state = *CONNECTION_STATE.lock().unwrap();
        if state == ConnectionState::Connected || state == ConnectionState::Connecting {
            return Err("Already connected or connecting".to_string());
        }
    }
    
    // Set state to connecting
    {
        let mut state = CONNECTION_STATE.lock().unwrap();
        *state = ConnectionState::Connecting;
    }
    
    // Initialize default table handlers
    init_default_table_handlers();
    
    // Build connection URL and database address
    let address = format!("{}/{}", params.host, params.database_name);
    
    // Build auth token if provided
    let identity = match &params.auth_token {
        Some(token) => {
            Identity::from_private_key_bytes(token.as_bytes().to_vec())
                .map_err(|e| {
                    let mut state = CONNECTION_STATE.lock().unwrap();
                    *state = ConnectionState::Disconnected;
                    format!("Invalid auth token: {}", e)
                })
                .ok()
        },
        None => None,
    };
    
    // Create client
    let client = sdk_client::Client::new(address, identity);
    
    // Register event handlers
    client.on_client_state_change(handle_client_state_change);
    client.on_subscription_applied(handle_subscription_applied);
    client.on_disconnect(handle_disconnect);
    client.on_subscription_failed(handle_subscription_failed);
    client.on_table_update(handle_table_update);
    client.on_reducer_call(handle_reducer_call);
    
    // Store client
    {
        let mut client_instance = CLIENT.lock().unwrap();
        *client_instance = Some(client.clone());
    }
    
    // Connect and subscribe
    let subscription = match tokio::runtime::Handle::try_current() {
        Ok(handle) => {
            // If we're in a tokio runtime, use that
            handle.block_on(client.subscribe())
                .map_err(|e| format!("Failed to subscribe: {}", e))?
        },
        Err(_) => {
            // If not, create a new runtime for this call
            let rt = tokio::runtime::Runtime::new()
                .map_err(|e| format!("Failed to create runtime: {}", e))?;
            rt.block_on(client.subscribe())
                .map_err(|e| format!("Failed to subscribe: {}", e))?
        }
    };
    
    // Store subscription
    {
        let mut sub = SUBSCRIPTION.lock().unwrap();
        *sub = Some(subscription);
    }
    
    info!("Connection to SpacetimeDB initiated");
    
    Ok(())
}

/// Disconnect from the SpacetimeDB server
pub fn disconnect_from_server() -> bool {
    // Check if connected
    {
        let state = *CONNECTION_STATE.lock().unwrap();
        if state != ConnectionState::Connected && state != ConnectionState::Connecting {
            return false;
        }
    }
    
    // Unsubscribe
    {
        let mut sub = SUBSCRIPTION.lock().unwrap();
        if let Some(subscription) = sub.take() {
            let client = CLIENT.lock().unwrap();
            if let Some(client) = &*client {
                let rt = match tokio::runtime::Runtime::new() {
                    Ok(rt) => rt,
                    Err(e) => {
                        error!("Failed to create runtime for disconnect: {}", e);
                        return false;
                    }
                };
                
                rt.block_on(subscription.unsubscribe(client));
                debug!("Unsubscribed from SpacetimeDB");
            }
        }
    }
    
    // Close client
    {
        let mut client = CLIENT.lock().unwrap();
        if let Some(c) = client.take() {
            // Client will be dropped here
            debug!("SpacetimeDB client closed");
        }
    }
    
    // Set state to disconnected
    {
        let mut state = CONNECTION_STATE.lock().unwrap();
        *state = ConnectionState::Disconnected;
    }
    
    // Clear client ID
    {
        let mut client_id = CLIENT_ID.lock().unwrap();
        *client_id = 0;
    }
    
    // Call the on_disconnected handler
    if let Some(handler) = &*ON_DISCONNECTED.lock().unwrap() {
        handler("Disconnected by client");
    }
    
    true
}

/// Disconnect from the SpacetimeDB server (alias for disconnect_from_server)
pub fn disconnect() -> bool {
    disconnect_from_server()
}

/// Check if connected to the server
pub fn is_connected() -> bool {
    let state = *CONNECTION_STATE.lock().unwrap();
    state == ConnectionState::Connected
}

/// Get the client ID
pub fn get_client_id() -> u64 {
    *CLIENT_ID.lock().unwrap()
}

/// Set event handlers
pub fn set_event_handlers(
    on_connected: impl Fn() + Send + 'static,
    on_disconnected: impl Fn(&str) + Send + 'static,
    on_error: impl Fn(&str) + Send + 'static,
    on_property_updated: impl Fn(ObjectId, &str, &str) + Send + 'static,
    on_object_created: impl Fn(ObjectId, &str, &str) + Send + 'static,
    on_object_destroyed: impl Fn(ObjectId) + Send + 'static,
) {
    // Set handlers
    {
        let mut handler = ON_CONNECTED.lock().unwrap();
        *handler = Some(Box::new(on_connected));
    }
    {
        let mut handler = ON_DISCONNECTED.lock().unwrap();
        *handler = Some(Box::new(on_disconnected));
    }
    {
        let mut handler = ON_ERROR.lock().unwrap();
        *handler = Some(Box::new(on_error));
    }
    {
        let mut handler = ON_PROPERTY_UPDATED.lock().unwrap();
        *handler = Some(Box::new(on_property_updated));
    }
    {
        let mut handler = ON_OBJECT_CREATED.lock().unwrap();
        *handler = Some(Box::new(on_object_created));
    }
    {
        let mut handler = ON_OBJECT_DESTROYED.lock().unwrap();
        *handler = Some(Box::new(on_object_destroyed));
    }
}

/// Send a property update to the server
pub fn send_property_update(
    object_id: ObjectId,
    property_name: &str,
    value_json: &str,
) -> Result<(), String> {
    // Check if connected
    if !is_connected() {
        return Err("Not connected to server".to_string());
    }
    
    // Get client
    let client_lock = CLIENT.lock().unwrap();
    let client = match &*client_lock {
        Some(c) => c,
        None => return Err("Client not available".to_string()),
    };
    
    // Get client_id
    let client_id = *CLIENT_ID.lock().unwrap();
    
    // Call the server reducer
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => return Err(format!("Failed to create runtime: {}", e)),
    };
    
    // Call set_object_property reducer
    let result = rt.block_on(client.call_reducer(
        "set_object_property", 
        (client_id, object_id, property_name, value_json)
    ));
    
    match result {
        Ok(status) => {
            match status {
                ReducerStatus::Committed => {
                    debug!("Property update committed: object_id={}, property={}", object_id, property_name);
                    Ok(())
                },
                ReducerStatus::Failed(e) => {
                    error!("Property update failed: {}", e);
                    Err(format!("Property update failed: {}", e))
                },
            }
        },
        Err(e) => {
            error!("Failed to call set_object_property reducer: {}", e);
            Err(format!("Failed to call property update: {}", e))
        }
    }
}

/// Send an RPC request to the server
pub fn send_rpc_request(request_json: &str) -> Result<(), String> {
    // Check if connected
    if !is_connected() {
        return Err("Not connected to server".to_string());
    }
    
    // Parse the request JSON
    let request: Value = match serde_json::from_str(request_json) {
        Ok(req) => req,
        Err(e) => return Err(format!("Invalid RPC request JSON: {}", e)),
    };
    
    // Extract the function name and arguments
    let function_name = match request.get("function") {
        Some(Value::String(name)) => name,
        _ => return Err("RPC request missing 'function' field".to_string()),
    };
    
    let object_id = match request.get("object_id") {
        Some(Value::Number(id)) => {
            match id.as_u64() {
                Some(id) => id,
                None => return Err("Invalid object_id in RPC request".to_string()),
            }
        },
        _ => return Err("RPC request missing 'object_id' field".to_string()),
    };
    
    let args_json = match request.get("args") {
        Some(args) => serde_json::to_string(args)
            .map_err(|e| format!("Failed to serialize RPC arguments: {}", e))?,
        None => return Err("RPC request missing 'args' field".to_string()),
    };
    
    // Get client
    let client_lock = CLIENT.lock().unwrap();
    let client = match &*client_lock {
        Some(c) => c,
        None => return Err("Client not available".to_string()),
    };
    
    // Get client_id
    let client_id = *CLIENT_ID.lock().unwrap();
    
    // Create runtime
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => return Err(format!("Failed to create runtime: {}", e)),
    };
    
    // Call execute_rpc reducer
    let result = rt.block_on(client.call_reducer(
        "execute_rpc", 
        (client_id, object_id, function_name, args_json)
    ));
    
    match result {
        Ok(status) => {
            match status {
                ReducerStatus::Committed => {
                    debug!("RPC call committed: function={}, object_id={}", function_name, object_id);
                    Ok(())
                },
                ReducerStatus::Failed(e) => {
                    error!("RPC call failed: {}", e);
                    Err(format!("RPC call failed: {}", e))
                },
            }
        },
        Err(e) => {
            error!("Failed to call execute_rpc reducer: {}", e);
            Err(format!("Failed to call RPC: {}", e))
        }
    }
}

/// Convert from SDK DisconnectReason to shared module's DisconnectReason
fn convert_disconnect_reason(reason: DisconnectReason) -> SharedDisconnectReason {
    match reason {
        DisconnectReason::ClientRequested => SharedDisconnectReason::ClientRequest,
        DisconnectReason::ServerShutdown => SharedDisconnectReason::ServerShutdown,
        DisconnectReason::ConnectionTimeout => SharedDisconnectReason::Timeout,
        DisconnectReason::AuthenticationFailure => SharedDisconnectReason::AuthFailure,
        DisconnectReason::NetworkError(err) => SharedDisconnectReason::NetworkError(err),
        DisconnectReason::ServerError(err) => SharedDisconnectReason::NetworkError(format!("Server error: {}", err)),
        _ => SharedDisconnectReason::Unknown,
    }
}

// ---- SpacetimeDB event handlers ----

/// Handle client state changes
fn handle_client_state_change(state: sdk_client::ClientState) {
    match state {
        sdk_client::ClientState::Connected => {
            debug!("SpacetimeDB client connected");
            
            // Update connection state
            {
                let mut conn_state = CONNECTION_STATE.lock().unwrap();
                *conn_state = ConnectionState::Connected;
            }
            
            // Get and store identity
            let client_lock = CLIENT.lock().unwrap();
            if let Some(client) = &*client_lock {
                if let Some(identity) = client.identity() {
                    let client_id_bytes = identity.as_bytes();
                    let mut client_id_u64: u64 = 0;
                    
                    // Convert identity bytes to u64 (use first 8 bytes)
                    for (i, &byte) in client_id_bytes.iter().take(8).enumerate() {
                        client_id_u64 |= (byte as u64) << (i * 8);
                    }
                    
                    // Store client ID
                    {
                        let mut client_id = CLIENT_ID.lock().unwrap();
                        *client_id = client_id_u64;
                    }
                    
                    info!("Connected to SpacetimeDB with client ID: {}", client_id_u64);
                }
            }
            
            // Call the on_connected handler
            if let Some(handler) = &*ON_CONNECTED.lock().unwrap() {
                handler();
            }
        },
        sdk_client::ClientState::Connecting => {
            debug!("SpacetimeDB client connecting");
            
            // Update connection state
            let mut state = CONNECTION_STATE.lock().unwrap();
            *state = ConnectionState::Connecting;
        },
        sdk_client::ClientState::Disconnecting => {
            debug!("SpacetimeDB client disconnecting");
        },
        sdk_client::ClientState::Disconnected => {
            debug!("SpacetimeDB client disconnected");
            
            // Update connection state
            {
                let mut state = CONNECTION_STATE.lock().unwrap();
                *state = ConnectionState::Disconnected;
            }
            
            // Call the on_disconnected handler
            if let Some(handler) = &*ON_DISCONNECTED.lock().unwrap() {
                handler("Disconnected from server");
            }
        },
    }
}

/// Handle subscription applied
fn handle_subscription_applied() {
    info!("SpacetimeDB subscription applied");
    
    // Update connection state
    {
        let mut state = CONNECTION_STATE.lock().unwrap();
        *state = ConnectionState::Connected;
    }
    
    info!("Connected to SpacetimeDB server");
    
    // Get client identity and ID
    {
        let client = CLIENT.lock().unwrap();
        if let Some(client) = &*client {
            if let Some(identity) = client.identity() {
                // Calculate client ID from identity
                let id_bytes = identity.as_bytes();
                let mut client_id: u64 = 0;
                
                // Use first 8 bytes of identity to create a u64 client ID
                for i in 0..std::cmp::min(8, id_bytes.len()) {
                    client_id = (client_id << 8) | (id_bytes[i] as u64);
                }
                
                // Store client ID
                let mut id = CLIENT_ID.lock().unwrap();
                *id = client_id;
                
                debug!("Client ID set to: {}", client_id);
            }
        }
    }
    
    // Log the property definition count after subscription
    debug!("Property definitions available after connection: {}", 
           crate::property::get_property_definition_count());
    
    // Call the on_connected handler
    if let Some(handler) = &*ON_CONNECTED.lock().unwrap() {
        handler();
    }
}

/// Handle disconnect
fn handle_disconnect(reason: DisconnectReason) {
    warn!("SpacetimeDB disconnected: {:?}", reason);
    
    // Update connection state
    {
        let mut state = CONNECTION_STATE.lock().unwrap();
        *state = ConnectionState::Disconnected;
    }
    
    // Clear client ID
    {
        let mut client_id = CLIENT_ID.lock().unwrap();
        *client_id = 0;
    }
    
    // Clear subscription
    {
        let mut sub = SUBSCRIPTION.lock().unwrap();
        *sub = None;
    }
    
    // Call the on_disconnected handler
    if let Some(handler) = &*ON_DISCONNECTED.lock().unwrap() {
        // Convert SDK DisconnectReason to a string using our shared type
        let shared_reason = convert_disconnect_reason(reason);
        let reason_str = match shared_reason {
            SharedDisconnectReason::ClientRequest => "Client requested disconnect",
            SharedDisconnectReason::ServerShutdown => "Server shutdown",
            SharedDisconnectReason::Timeout => "Connection timeout",
            SharedDisconnectReason::AuthFailure => "Authentication failure",
            SharedDisconnectReason::NetworkError(err) => &err,
            SharedDisconnectReason::Kicked(msg) => &msg,
            SharedDisconnectReason::Unknown => "Unknown reason",
        };
        
        handler(reason_str);
    }
}

/// Handle subscription failed
fn handle_subscription_failed(error: String) {
    error!("SpacetimeDB subscription failed: {}", error);
    
    // Update connection state
    {
        let mut state = CONNECTION_STATE.lock().unwrap();
        *state = ConnectionState::Failed;
    }
    
    // Call the on_error handler
    if let Some(handler) = &*ON_ERROR.lock().unwrap() {
        handler(&format!("Subscription failed: {}", error));
    }
}

/// Process table updates from the server
fn handle_table_update(update: TableUpdate) {
    // Forward to the appropriate handler based on table name
    if let Err(err) = process_table_update(&update) {
        error!("Error processing table update: {}", err);
        if let Some(handler) = &*ON_ERROR.lock().unwrap() {
            handler(&format!("Error processing table update: {}", err));
        }
    }
}

/// Handle a reducer call (RPC) from the server
fn handle_reducer_call(func_name: String, arg_bytes: Vec<u8>, _caller_identity: Option<Identity>) {
    // Check if there are the right number of bytes
    if arg_bytes.len() < 8 {
        error!("Invalid reducer call - insufficient bytes in argument");
        return;
    }
    
    // First 8 bytes should be the object ID
    let object_id_bytes = &arg_bytes[0..8];
    let object_id = u64::from_le_bytes([
        object_id_bytes[0], object_id_bytes[1], object_id_bytes[2], object_id_bytes[3],
        object_id_bytes[4], object_id_bytes[5], object_id_bytes[6], object_id_bytes[7],
    ]);
    
    // The rest should be a JSON string for the arguments
    match String::from_utf8(arg_bytes[8..].to_vec()) {
        Ok(args_json) => {
            // Call the appropriate RPC handler
            if let Err(err) = crate::rpc::handle_server_call(object_id, &func_name, &args_json) {
                error!("Error handling RPC call {}: {}", func_name, err);
                if let Some(handler) = &*ON_ERROR.lock().unwrap() {
                    handler(&format!("Error handling RPC call {}: {}", func_name, err));
                }
            }
        },
        Err(err) => {
            error!("Invalid UTF-8 in RPC arguments: {}", err);
            if let Some(handler) = &*ON_ERROR.lock().unwrap() {
                handler(&format!("Invalid UTF-8 in RPC arguments: {}", err));
            }
        }
    }
}

// ---- Table update handlers ----

/// Process a table update from SpacetimeDB
pub fn process_table_update(update: &TableUpdate) -> Result<(), String> {
    let table_name = &update.table_name;
    debug!("Processing update for table: {}", table_name);
    
    // Look for registered handlers
    let handlers = TABLE_HANDLERS.lock().unwrap();
    
    let mut handled = false;
    for handler in handlers.iter() {
        if handler.table_name == table_name {
            if let Err(e) = (handler.handler)(update) {
                error!("Error handling update for table {}: {}", table_name, e);
                return Err(e);
            }
            handled = true;
        }
    }
    
    if !handled {
        debug!("No handler registered for table: {}", table_name);
    }
    
    Ok(())
}

/// Handle updates to the ObjectInstance table
fn handle_object_instance_update(update: &TableUpdate) -> Result<(), String> {
    // Process all rows in the update
    for row in &update.table_row_operations {
        match row.operation {
            TableOp::Insert | TableOp::Update => {
                // Extract object data
                if let Some(row_data) = &row.row {
                    // Extract object ID
                    let object_id = match row_data.get("id") {
                        Some(Value::U64(id)) => *id,
                        _ => return Err("Missing or invalid object ID".to_string()),
                    };
                    
                    // Extract class name
                    let class_name = match row_data.get("class_name") {
                        Some(Value::String(name)) => name.clone(),
                        _ => return Err("Missing or invalid class name".to_string()),
                    };
                    
                    // Extract lifecycle state
                    let state = match row_data.get("state") {
                        Some(Value::U8(state)) => *state,
                        _ => 0, // Default to 0 (Initializing) if not found
                    };
                    
                    if state == 3 { // ObjectLifecycleState::Destroyed
                        // Object is destroyed, trigger destruction handler
                        if let Some(handler) = &*ON_OBJECT_DESTROYED.lock().unwrap() {
                            handler(object_id);
                        }
                    } else if row.operation == TableOp::Insert {
                        // New object, trigger creation handler
                        if let Some(handler) = &*ON_OBJECT_CREATED.lock().unwrap() {
                            // For now, use empty JSON for initial properties
                            // In a real implementation, we would query initial properties
                            handler(object_id, &class_name, "{}");
                        }
                    }
                }
            },
            TableOp::Delete => {
                // Find the object ID and trigger destruction
                if let Some(row_data) = &row.row {
                    if let Some(Value::U64(object_id)) = row_data.get("id") {
                        if let Some(handler) = &*ON_OBJECT_DESTROYED.lock().unwrap() {
                            handler(*object_id);
                        }
                    }
                }
            },
        }
    }
    
    Ok(())
}

/// Handle updates to the ObjectProperty table
fn handle_object_property_update(update: &TableUpdate) -> Result<(), String> {
    for row in &update.table_row_operations {
        if row.operation != TableOp::Delete {
            if let Some(row_data) = &row.row {
                // Extract object ID
                let object_id = match row_data.get("object_id") {
                    Some(Value::U64(id)) => *id,
                    _ => return Err("Missing or invalid object ID".to_string()),
                };
                
                // Extract property name
                let property_name = match row_data.get("name") {
                    Some(Value::String(name)) => name.clone(),
                    _ => return Err("Missing or invalid property name".to_string()),
                };
                
                // Extract property value
                let value_json = match row_data.get("value") {
                    Some(value) => serde_json::to_string(value)
                        .unwrap_or_else(|_| "null".to_string()),
                    None => "null".to_string(),
                };
                
                // Trigger property updated handler
                if let Some(handler) = &*ON_PROPERTY_UPDATED.lock().unwrap() {
                    handler(object_id, &property_name, &value_json);
                }
            }
        }
    }
    
    Ok(())
}

/// Process updates to object transforms in a transform table update
fn handle_object_transform_update(update: &TableUpdate) -> Result<(), String> {
    for row in &update.table_row_operations {
        if let Some(row_data) = &row.row {
            // Extract object ID
            let object_id = match row_data.get("object_id") {
                Some(Value::U64(id)) => *id,
                _ => continue,
            };
            
            match row.operation {
                TableOp::Insert | TableOp::Update => {
                    // Extract transform components
                    let location_x = row_data.get("location_x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                    let location_y = row_data.get("location_y").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                    let location_z = row_data.get("location_z").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                    
                    let rotation_x = row_data.get("rotation_x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                    let rotation_y = row_data.get("rotation_y").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                    let rotation_z = row_data.get("rotation_z").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                    let rotation_w = row_data.get("rotation_w").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
                    
                    let scale_x = row_data.get("scale_x").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
                    let scale_y = row_data.get("scale_y").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
                    let scale_z = row_data.get("scale_z").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
                    
                    // Update the object's transform
                    let location = stdb_shared::types::Vector3 {
                        x: location_x,
                        y: location_y,
                        z: location_z,
                    };
                    
                    let rotation = stdb_shared::types::Quat {
                        x: rotation_x,
                        y: rotation_y,
                        z: rotation_z,
                        w: rotation_w,
                    };
                    
                    let scale = stdb_shared::types::Vector3 {
                        x: scale_x,
                        y: scale_y,
                        z: scale_z,
                    };
                    
                    // Update the object's transform
                    if let Err(err) = crate::object::update_transform(object_id, Some(location), Some(rotation), Some(scale)) {
                        warn!("Failed to update transform for object {}: {}", object_id, err);
                    }
                },
                _ => {}
            }
        }
    }
    
    Ok(())
}

/// Handle updates to the PropertyDefinitionTable
fn handle_property_definition_update(update: &TableUpdate) -> Result<(), String> {
    // Process all rows in the update
    for row in &update.table_row_operations {
        if let Some(row_data) = &row.row {
            match row.operation {
                TableOp::Insert | TableOp::Update => {
                    // Extract fields from the row
                    let class_name = match row_data.get("class_name") {
                        Some(Value::String(s)) => s,
                        _ => continue,
                    };
                    
                    let prop_name = match row_data.get("property_name") {
                        Some(Value::String(s)) => s,
                        _ => continue,
                    };
                    
                    let prop_type_str = match row_data.get("property_type") {
                        Some(Value::String(s)) => s,
                        _ => continue,
                    };
                    
                    let replicated = match row_data.get("replicated") {
                        Some(Value::Bool(b)) => *b,
                        _ => false,
                    };
                    
                    let readonly = match row_data.get("readonly") {
                        Some(Value::Bool(b)) => *b,
                        _ => false,
                    };
                    
                    // Parse property type (removing the "PropertyType::" prefix if present)
                    let type_str = prop_type_str.replace("PropertyType::", "");
                    let prop_type = match type_str.as_str() {
                        "Bool" => stdb_shared::property::PropertyType::Bool,
                        "Byte" => stdb_shared::property::PropertyType::Byte,
                        "Int32" => stdb_shared::property::PropertyType::Int32,
                        "Int64" => stdb_shared::property::PropertyType::Int64,
                        "UInt32" => stdb_shared::property::PropertyType::UInt32,
                        "UInt64" => stdb_shared::property::PropertyType::UInt64,
                        "Float" => stdb_shared::property::PropertyType::Float,
                        "Double" => stdb_shared::property::PropertyType::Double,
                        "String" => stdb_shared::property::PropertyType::String,
                        "Vector" => stdb_shared::property::PropertyType::Vector,
                        "Rotator" => stdb_shared::property::PropertyType::Rotator,
                        "Quat" => stdb_shared::property::PropertyType::Quat,
                        "Transform" => stdb_shared::property::PropertyType::Transform,
                        "Color" => stdb_shared::property::PropertyType::Color,
                        "ObjectReference" => stdb_shared::property::PropertyType::ObjectReference,
                        "ClassReference" => stdb_shared::property::PropertyType::ClassReference,
                        "Array" => stdb_shared::property::PropertyType::Array,
                        "Map" => stdb_shared::property::PropertyType::Map,
                        "Set" => stdb_shared::property::PropertyType::Set,
                        "Name" => stdb_shared::property::PropertyType::Name,
                        "Text" => stdb_shared::property::PropertyType::Text,
                        "Custom" => stdb_shared::property::PropertyType::Custom,
                        "None" => stdb_shared::property::PropertyType::None,
                        _ => {
                            warn!("Unknown property type: {}", type_str);
                            stdb_shared::property::PropertyType::None
                        }
                    };
                    
                    // Register the property definition
                    crate::property::register_property_definition(
                        class_name,
                        prop_name,
                        prop_type,
                        replicated
                    );
                    
                    trace!("Registered property definition from server: {}.{} (type: {:?}, replicated: {})", 
                        class_name, prop_name, prop_type, replicated);
                },
                TableOp::Delete => {
                    // For now, we don't handle property definition deletion
                },
                _ => {}
            }
        }
    }
    
    // Log number of property definitions we have now
    debug!("Total property definitions registered: {}", crate::property::get_property_definition_count());
    
    Ok(())
} 