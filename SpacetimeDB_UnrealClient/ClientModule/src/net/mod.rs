//! # Network Module
//!
//! Handles communication with the SpacetimeDB server, including connection management
//! and message passing.

use stdb_shared::object::ObjectId;
use stdb_shared::connection::{ConnectionState, ConnectionParams, DisconnectReason as SharedDisconnectReason};

// Update to match current SDK structure
use spacetimedb;

// Use our custom TableUpdate from the property module
use crate::property::TableUpdate;

use std::sync::Mutex;
use once_cell::sync::Lazy;
use log::{info, debug, error};

// Global client state
static CLIENT_STATE: Lazy<Mutex<ConnectionState>> = Lazy::new(|| Mutex::new(ConnectionState::Disconnected));
static CLIENT_ID: Lazy<Mutex<u64>> = Lazy::new(|| Mutex::new(0));

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

/// Connect to the SpacetimeDB server
pub fn connect(_params: ConnectionParams) -> Result<(), String> {
    // Already connected or connecting
    {
        let state = *CLIENT_STATE.lock().unwrap();
        if state == ConnectionState::Connected || state == ConnectionState::Connecting {
            return Err("Already connected or connecting".to_string());
        }
    }
    
    // Set state to connecting
    {
        let mut state = CLIENT_STATE.lock().unwrap();
        *state = ConnectionState::Connecting;
    }
    
    // Simulation mode - we'll set up a fake connection
    // In a real implementation, this would actually connect to SpacetimeDB
    {
        let mut state = CLIENT_STATE.lock().unwrap();
        *state = ConnectionState::Connected;
    }
    
    // Set a fake client ID
    {
        let mut client_id = CLIENT_ID.lock().unwrap();
        *client_id = 12345; // Fake client ID for testing
    }
    
    // Call the on_connected handler
    if let Some(handler) = &*ON_CONNECTED.lock().unwrap() {
        handler();
    }
    
    info!("Connected to SpacetimeDB (simulated)");
    
    Ok(())
}

/// Disconnect from the SpacetimeDB server
pub fn disconnect() -> bool {
    // Check if connected
    {
        let state = *CLIENT_STATE.lock().unwrap();
        if state != ConnectionState::Connected && state != ConnectionState::Connecting {
            return false;
        }
    }
    
    // Set state to disconnected
    {
        let mut state = CLIENT_STATE.lock().unwrap();
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

/// Check if connected to the server
pub fn is_connected() -> bool {
    let state = *CLIENT_STATE.lock().unwrap();
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

/// Convert from SDK DisconnectReason to our shared DisconnectReason
fn convert_disconnect_reason(reason: String) -> SharedDisconnectReason {
    match reason.as_str() {
        "ClosedByClient" => SharedDisconnectReason::ClientRequest,
        "ClosedByServer" => SharedDisconnectReason::ServerShutdown,
        "ConnectionError" => SharedDisconnectReason::NetworkError(reason),
        "Timeout" => SharedDisconnectReason::Timeout,
        _ => SharedDisconnectReason::Unknown,
    }
}

/// Process a table update
pub fn process_table_update(update: &TableUpdate) -> Result<(), String> {
    // Find handlers for this table
    let handlers = TABLE_HANDLERS.lock().unwrap();
    let matching_handlers: Vec<_> = handlers
        .iter()
        .filter(|h| h.table_name == update.table_name)
        .collect();
    
    // Call each handler
    for handler in matching_handlers {
        if let Err(e) = (handler.handler)(update) {
            error!("Error handling table update for {}: {}", update.table_name, e);
            return Err(e);
        }
    }
    
    Ok(())
}

/// Handle an object instance update
fn handle_object_instance_update(_update: &TableUpdate) -> Result<(), String> {
    // Implementation stub - would process object instance updates
    Ok(())
}

/// Handle an object property update
fn handle_object_property_update(_update: &TableUpdate) -> Result<(), String> {
    // Implementation stub - would process property updates
    Ok(())
}

/// Handle an object transform update
fn handle_object_transform_update(_update: &TableUpdate) -> Result<(), String> {
    // Implementation stub - would process transform updates
    Ok(())
}

/// Handle a property definition update
fn handle_property_definition_update(update: &TableUpdate) -> Result<(), String> {
    // Implementation stub - would process property definition updates
    // This would typically delegate to the property module
    crate::property::handle_property_definition_update(update)
}

/// Send an RPC request to the server
pub fn send_rpc_request(request_json: &str) -> Result<(), String> {
    // Check if connected
    if !is_connected() {
        return Err("Not connected to server".to_string());
    }
    
    // In a real implementation, this would send the request to the SpacetimeDB server
    debug!("Sending RPC request: {}", request_json);
    
    // Simulate success for now
    Ok(())
}

/// Send a property update to the server
pub fn send_property_update(object_id: ObjectId, property_name: &str, value_json: &str) -> Result<(), String> {
    // Check if connected
    if !is_connected() {
        return Err("Not connected to server".to_string());
    }
    
    // In a real implementation, this would send the property update to the SpacetimeDB server
    debug!("Sending property update for object {}, property {}: {}", 
           object_id, property_name, value_json);
    
    // Simulate success for now
    Ok(())
} 