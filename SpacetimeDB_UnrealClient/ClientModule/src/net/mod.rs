//! # Network Module
//!
//! Handles communication with the SpacetimeDB server, including connection management
//! and message passing.

use stdb_shared::object::ObjectId;
use spacetimedb_sdk::{
    Address, Client, Identity, ReducerCallError, subscribe::SubscriptionHandle
};

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use once_cell::sync::Lazy;

/// State of the connection to the server
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Not connected to the server
    Disconnected,
    
    /// Attempting to connect to the server
    Connecting,
    
    /// Connected to the server
    Connected,
    
    /// Connection failed
    Failed,
}

/// Connection parameters for the SpacetimeDB server
#[derive(Debug, Clone)]
pub struct ConnectionParams {
    /// Host address (e.g., "https://example.com")
    pub host: String,
    
    /// Database name
    pub database_name: String,
    
    /// Optional authentication token
    pub auth_token: Option<String>,
}

/// Connection information
#[derive(Debug, Clone)]
pub struct ClientConnection {
    /// Current connection state
    pub state: ConnectionState,
    
    /// Client ID assigned by the server
    pub client_id: u64,
    
    /// Connection parameters
    pub params: ConnectionParams,
}

// Global client state
static CLIENT: Lazy<Mutex<Option<Client>>> = Lazy::new(|| Mutex::new(None));
static CONNECTION_STATE: Lazy<Mutex<ConnectionState>> = Lazy::new(|| Mutex::new(ConnectionState::Disconnected));
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
    
    // Build connection URL
    let url = format!("{}/{}", params.host, params.database_name);
    
    // This is just a placeholder implementation
    // In a real implementation, we would use spacetimedb_sdk to connect
    
    // Simulate successful connection
    {
        let mut state = CONNECTION_STATE.lock().unwrap();
        *state = ConnectionState::Connected;
    }
    
    // Assign a client ID
    {
        let mut client_id = CLIENT_ID.lock().unwrap();
        *client_id = rand::random::<u64>();
    }
    
    // Call the on_connected handler
    if let Some(handler) = &*ON_CONNECTED.lock().unwrap() {
        handler();
    }
    
    Ok(())
}

/// Disconnect from the SpacetimeDB server
pub fn disconnect() -> bool {
    // Check if connected
    {
        let state = *CONNECTION_STATE.lock().unwrap();
        if state != ConnectionState::Connected {
            return false;
        }
    }
    
    // Set state to disconnected
    {
        let mut state = CONNECTION_STATE.lock().unwrap();
        *state = ConnectionState::Disconnected;
    }
    
    // Call the on_disconnected handler
    if let Some(handler) = &*ON_DISCONNECTED.lock().unwrap() {
        handler("Disconnected by client");
    }
    
    // Clear client ID
    {
        let mut client_id = CLIENT_ID.lock().unwrap();
        *client_id = 0;
    }
    
    // Clear client
    {
        let mut client = CLIENT.lock().unwrap();
        *client = None;
    }
    
    true
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
    
    // In a real implementation, we would call the server reducer here
    // For now, just log the update
    println!(
        "Would send property update: object_id={}, property_name={}, value={}",
        object_id, property_name, value_json
    );
    
    Ok(())
}

/// Send an RPC request to the server
pub fn send_rpc_request(request_json: &str) -> Result<(), String> {
    // Check if connected
    if !is_connected() {
        return Err("Not connected to server".to_string());
    }
    
    // In a real implementation, we would use spacetimedb_sdk to call a reducer
    info!("Sending RPC request: {}", request_json);
    
    // For now, simply log the request and return success
    debug!("RPC request sent successfully");
    Ok(())
} 