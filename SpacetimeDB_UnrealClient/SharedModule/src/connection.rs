//! # Connection Types
//!
//! Shared connection-related types used by both client and server.
//! This module contains definitions related to client connections,
//! connection states, and authentication.

use serde::{Serialize, Deserialize};

/// State of the connection to the server
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// Basic client identification information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientIdentity {
    /// Unique client identifier
    pub client_id: u64,
    
    /// Optional client display name
    pub display_name: Option<String>,
    
    /// Whether this client has admin privileges
    pub is_admin: bool,
}

/// Connection parameters for the SpacetimeDB server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionParams {
    /// Host address (e.g., "https://example.com")
    pub host: String,
    
    /// Database name
    pub database_name: String,
    
    /// Optional authentication token
    pub auth_token: Option<String>,
}

/// Connection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConnection {
    /// Current connection state
    pub state: ConnectionState,
    
    /// Client ID assigned by the server
    pub client_id: u64,
    
    /// Connection parameters
    pub params: ConnectionParams,
    
    /// When the client connected (timestamp)
    pub connected_at: u64,
}

/// The reason for a disconnection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisconnectReason {
    /// Normal disconnection by client request
    ClientRequest,
    
    /// Server shutting down
    ServerShutdown,
    
    /// Connection timeout
    Timeout,
    
    /// Authentication failure
    AuthFailure,
    
    /// Network error
    NetworkError(String),
    
    /// Kicked by server or admin
    Kicked(String),
    
    /// Unknown reason
    Unknown,
} 