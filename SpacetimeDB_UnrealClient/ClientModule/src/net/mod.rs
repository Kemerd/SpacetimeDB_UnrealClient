//! # Network Module
//!
//! Handles communication with the SpacetimeDB server, including connection management
//! and message passing.

use stdb_shared::object::ObjectId;
use stdb_shared::connection::{ConnectionState, ConnectionParams, DisconnectReason as SharedDisconnectReason};

// Use our custom TableUpdate from the property module
use crate::property::TableUpdate;

use std::sync::Mutex;
use once_cell::sync::Lazy;
use log::{info, debug, error};
use serde_json;
use std::collections::HashMap;

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

/// Get the client identity as a hex string
pub fn get_client_identity_hex() -> Option<String> {
    if !is_connected() {
        return None;
    }
    
    // Get the client ID and format it as a hex string
    let client_id = get_client_id();
    if client_id == 0 {
        return None;
    }
    
    // Convert the client ID to a hex string
    Some(format!("{:016x}", client_id))
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

/// Add a new function for sending WebSocket messages
/// Send a message to the SpacetimeDB server
fn send_message(message: &str) -> Result<(), String> {
    // Check if connected
    if !is_connected() {
        return Err("Not connected to server".to_string());
    }
    
    // In a real implementation, this would use a WebSocket or other transport
    // For simulation purposes, just log the message
    debug!("Sending message to server: {}", message);
    
    // Simulated network delays and processing
    // In a real implementation, this would be asynchronous
    
    // Notify any error handlers in case of network errors
    if message.len() > 10000 {
        // Simulate a network error for very large messages
        if let Some(handler) = &*ON_ERROR.lock().unwrap() {
            handler("Network error: Message too large");
        }
        return Err("Message too large".to_string());
    }
    
    // Simulate success
    Ok(())
}

/// Send a subscription request to the server
fn send_subscription_request(table_name: &str) -> Result<(), String> {
    // Create the subscription message
    let subscription_message = serde_json::json!({
        "type": "subscribe",
        "table": table_name,
        "client_id": get_client_id()
    });
    
    // Convert to string
    let message_str = match serde_json::to_string(&subscription_message) {
        Ok(s) => s,
        Err(e) => return Err(format!("Failed to serialize subscription message: {}", e))
    };
    
    // Send the message using our transport function
    send_message(&message_str)
}

/// Send an RPC request to the server
pub fn send_rpc_request(request_json: &str) -> Result<(), String> {
    // Wrap the request in an RPC message envelope
    let rpc_message = serde_json::json!({
        "type": "rpc",
        "payload": serde_json::from_str::<serde_json::Value>(request_json).unwrap_or(serde_json::json!({})),
        "client_id": get_client_id()
    });
    
    // Serialize the message
    let message_str = match serde_json::to_string(&rpc_message) {
        Ok(s) => s,
        Err(e) => return Err(format!("Failed to serialize RPC message: {}", e))
    };
    
    // Send the message
    send_message(&message_str)
}

/// Send a property update to the server
pub fn send_property_update(object_id: ObjectId, property_name: &str, value_json: &str) -> Result<(), String> {
    // Create the property update message
    let update_message = serde_json::json!({
        "type": "property_update",
        "object_id": object_id,
        "property": property_name,
        "value": serde_json::from_str::<serde_json::Value>(value_json).unwrap_or(serde_json::json!(null)),
        "client_id": get_client_id()
    });
    
    // Serialize the message
    let message_str = match serde_json::to_string(&update_message) {
        Ok(s) => s,
        Err(e) => return Err(format!("Failed to serialize property update message: {}", e))
    };
    
    // Send the message
    send_message(&message_str)
}

/// Get a list of currently subscribed tables
pub fn get_subscribed_tables() -> Vec<String> {
    static SUBSCRIBED_TABLES: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));
    let subscribed = SUBSCRIBED_TABLES.lock().unwrap();
    subscribed.clone()
}

/// Process incoming data from the server
pub fn process_incoming_data(data: &str) -> Result<(), String> {
    // Parse the incoming data
    let parsed: serde_json::Value = match serde_json::from_str(data) {
        Ok(v) => v,
        Err(e) => return Err(format!("Failed to parse incoming data: {}", e))
    };
    
    // Get the message type
    let msg_type = match parsed.get("type") {
        Some(serde_json::Value::String(t)) => t.as_str(),
        _ => return Err("Missing or invalid message type".to_string())
    };
    
    // Process based on message type
    match msg_type {
        "table_update" => {
            // Extract the table name
            let table_name = match parsed.get("table") {
                Some(serde_json::Value::String(t)) => t,
                _ => return Err("Missing or invalid table name in table update".to_string())
            };
            
            // Extract the row operations
            let row_operations = match parsed.get("operations") {
                Some(serde_json::Value::Array(ops)) => {
                    // Convert each operation to a TableRowOperation
                    ops.iter().map(|op| {
                        // Default to Insert operation
                        let operation = match op.get("op") {
                            Some(serde_json::Value::String(op_str)) => {
                                match op_str.as_str() {
                                    "insert" => crate::property::TableOp::Insert,
                                    "update" => crate::property::TableOp::Update,
                                    "delete" => crate::property::TableOp::Delete,
                                    _ => crate::property::TableOp::Insert,
                                }
                            },
                            _ => crate::property::TableOp::Insert,
                        };
                        
                        // Extract row data
                        let row_data = match op.get("row") {
                            Some(serde_json::Value::Object(obj)) => {
                                // Convert to HashMap<String, Value>
                                let mut row_map = HashMap::new();
                                for (k, v) in obj {
                                    row_map.insert(k.clone(), v.clone());
                                }
                                Some(row_map)
                            },
                            _ => None,
                        };
                        
                        crate::property::TableRowOperation {
                            operation,
                            row: row_data,
                        }
                    }).collect()
                },
                _ => Vec::new()
            };
            
            // Create a TableUpdate
            let update = TableUpdate {
                table_name: table_name.to_string(),
                table_row_operations: row_operations,
            };
            
            // Process the update
            process_table_update(&update)
        },
        "connection_status" => {
            // Handle connection status updates
            Ok(())
        },
        "error" => {
            // Handle error messages
            let error_msg = match parsed.get("message") {
                Some(serde_json::Value::String(m)) => m,
                _ => "Unknown error from server"
            };
            
            // Notify error handlers
            if let Some(handler) = &*ON_ERROR.lock().unwrap() {
                handler(error_msg);
            }
            
            // Return the error
            Err(format!("Server error: {}", error_msg))
        },
        _ => {
            // Unknown message type
            Err(format!("Unknown message type: {}", msg_type))
        }
    }
}

/// Unsubscribe from tables
pub fn unsubscribe_from_tables(table_names: &[String]) -> Result<(), String> {
    // Check if connected
    if !is_connected() {
        return Err("Not connected to server".to_string());
    }
    
    // Track the subscribed tables
    static SUBSCRIBED_TABLES: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));
    
    // Remove tables from subscription list
    {
        let mut subscribed = SUBSCRIBED_TABLES.lock().unwrap();
        
        for table_name in table_names {
            // Skip if not subscribed
            if !subscribed.contains(table_name) {
                debug!("Table not subscribed: {}", table_name);
                continue;
            }
            
            // Remove from subscription list
            subscribed.retain(|t| t != table_name);
            
            // Send the unsubscription request
            if let Err(e) = send_unsubscription_request(table_name) {
                error!("Failed to send unsubscription request for table {}: {}", table_name, e);
                // Continue with other tables even if one fails
            } else {
                info!("Unsubscribed from table: {}", table_name);
            }
        }
    }
    
    // Return success
    Ok(())
}

/// Send an unsubscription request to the server
fn send_unsubscription_request(table_name: &str) -> Result<(), String> {
    // Create the unsubscription message
    let unsubscription_message = serde_json::json!({
        "type": "unsubscribe",
        "table": table_name,
        "client_id": get_client_id()
    });
    
    // Convert to string
    let message_str = match serde_json::to_string(&unsubscription_message) {
        Ok(s) => s,
        Err(e) => return Err(format!("Failed to serialize unsubscription message: {}", e))
    };
    
    // Send the message
    send_message(&message_str)
}

/// Subscribe to a set of tables in the SpacetimeDB database
pub fn subscribe_to_tables(table_names: &[String]) -> Result<(), String> {
    // Check if connected
    if !is_connected() {
        return Err("Not connected to server".to_string());
    }
    
    // Track the subscribed tables
    static SUBSCRIBED_TABLES: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));
    
    // Add tables to subscription list and register any necessary handlers
    {
        let mut subscribed = SUBSCRIBED_TABLES.lock().unwrap();
        
        for table_name in table_names {
            // Skip if already subscribed
            if subscribed.contains(table_name) {
                debug!("Table already subscribed: {}", table_name);
                continue;
            }
            
            // Add to subscription list
            subscribed.push(table_name.clone());
            
            // Register default handler if none exists
            let table_handlers = TABLE_HANDLERS.lock().unwrap();
            let has_handler = table_handlers.iter().any(|h| &h.table_name == table_name);
            
            if !has_handler {
                drop(table_handlers); // Release the lock before the next call
                
                // Register a default handler that just logs updates
                register_table_handler(table_name, move |update| {
                    debug!("Received update for table '{}' with {} operations", 
                           update.table_name, update.table_row_operations.len());
                    Ok(())
                });
            }
            
            // Send the actual subscription request to the server
            if let Err(e) = send_subscription_request(table_name) {
                error!("Failed to send subscription request for table {}: {}", table_name, e);
                // Continue with other tables even if one fails
            } else {
                info!("Subscribed to table: {}", table_name);
            }
        }
    }
    
    // Return success
    Ok(())
} 