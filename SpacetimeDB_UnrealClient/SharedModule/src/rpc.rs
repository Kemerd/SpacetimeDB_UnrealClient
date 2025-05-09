//! # RPC Types
//!
//! Shared RPC-related types used by both client and server.
//! This module contains definitions for remote procedure calls,
//! including function types, call formats, and error types.

use serde::{Serialize, Deserialize};
use crate::object::ObjectId;

/// Types of RPC calls
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RpcType {
    /// Server to client (implementation on client)
    Client,
    
    /// Client to server (implementation on server)
    Server,
    
    /// Multicast to all clients from server
    Multicast,
    
    /// Server to owner client only
    OwnerOnly,
}

/// Status of an RPC call
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RpcStatus {
    /// Call succeeded
    Success,
    
    /// Call failed
    Failed,
    
    /// Call is pending
    Pending,
    
    /// Call was rejected
    Rejected,
}

/// Error types for RPC calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RpcError {
    /// Function not found
    FunctionNotFound,
    
    /// Invalid arguments
    InvalidArguments(String),
    
    /// Object not found
    ObjectNotFound(ObjectId),
    
    /// Permission denied
    PermissionDenied,
    
    /// Internal server error
    InternalError(String),
    
    /// Network error
    NetworkError(String),
    
    /// Timeout
    Timeout,
    
    /// Function execution failed
    ExecutionFailed(String),
}

/// An RPC call request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcCall {
    /// ID of the object the function is called on
    pub object_id: ObjectId,
    
    /// Function name
    pub function_name: String,
    
    /// Arguments as JSON string
    pub arguments_json: String,
    
    /// Type of the RPC call
    pub rpc_type: RpcType,
    
    /// Optional call ID for tracking responses
    pub call_id: Option<u64>,
}

/// Response to an RPC call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    /// Original call ID
    pub call_id: u64,
    
    /// Status of the call
    pub status: RpcStatus,
    
    /// Result as JSON string (if successful)
    pub result_json: Option<String>,
    
    /// Error (if failed)
    pub error: Option<RpcError>,
}

/// RPC function registration info (shared between client/server)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcFunctionInfo {
    /// Function name
    pub name: String,
    
    /// Class the function belongs to
    pub class_name: String,
    
    /// Type of RPC
    pub rpc_type: RpcType,
    
    /// Whether the function is reliable (vs unreliable)
    pub is_reliable: bool,
} 