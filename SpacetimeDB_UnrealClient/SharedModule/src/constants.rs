//! # Shared Constants
//!
//! Constants used across both client and server modules.
//! 
//! These constants serve as compile-time defaults but can be overridden
//! at runtime via environment variables or configuration settings.

use std::env;
use std::str::FromStr;

/// Version of the SpacetimeDB Unreal Client
pub const STDB_CLIENT_VERSION: &str = "0.1.0";

/// Maximum supported number of objects
pub const MAX_OBJECTS: usize = 100_000;

/// Helper function to get a configured value from environment or use default
pub fn get_config<T: FromStr>(env_var: &str, default: T) -> T {
    match env::var(env_var) {
        Ok(val) => match val.parse::<T>() {
            Ok(parsed) => parsed,
            Err(_) => default,
        },
        Err(_) => default,
    }
}

/// Get the maximum number of objects (can be overridden with SPACETIME_MAX_OBJECTS)
pub fn get_max_objects() -> usize {
    get_config("SPACETIME_MAX_OBJECTS", MAX_OBJECTS)
}

/// Replication constants
pub mod replication {
    use super::get_config;

    /// Default replication interval (in seconds)
    pub const DEFAULT_REPLICATION_INTERVAL: f32 = 0.1;

    /// Maximum properties per replication frame
    pub const MAX_PROPERTIES_PER_FRAME: usize = 1000;

    /// Maximum objects per replication frame
    pub const MAX_OBJECTS_PER_FRAME: usize = 100;
    
    /// Get the replication interval (can be overridden with SPACETIME_REPLICATION_INTERVAL)
    pub fn get_replication_interval() -> f32 {
        get_config("SPACETIME_REPLICATION_INTERVAL", DEFAULT_REPLICATION_INTERVAL)
    }
    
    /// Get max properties per frame (can be overridden with SPACETIME_MAX_PROPERTIES_PER_FRAME)
    pub fn get_max_properties_per_frame() -> usize {
        get_config("SPACETIME_MAX_PROPERTIES_PER_FRAME", MAX_PROPERTIES_PER_FRAME)
    }
    
    /// Get max objects per frame (can be overridden with SPACETIME_MAX_OBJECTS_PER_FRAME)
    pub fn get_max_objects_per_frame() -> usize {
        get_config("SPACETIME_MAX_OBJECTS_PER_FRAME", MAX_OBJECTS_PER_FRAME)
    }
}

/// Object system constants
pub mod object {
    /// Root object class name
    pub const ROOT_OBJECT_CLASS: &str = "Object";
    
    /// Actor base class name
    pub const ACTOR_BASE_CLASS: &str = "Actor";
    
    /// Component base class name
    pub const COMPONENT_BASE_CLASS: &str = "ActorComponent";
    
    /// Reserved object IDs (0-999)
    pub const RESERVED_OBJECT_ID_MAX: u64 = 999;
    
    /// Special object ID indicating "no object"
    pub const NULL_OBJECT_ID: u64 = 0;
}

/// Network constants
pub mod network {
    use super::get_config;

    /// Default connection timeout (in seconds)
    pub const DEFAULT_TIMEOUT: f32 = 30.0;
    
    /// Maximum number of reconnection attempts
    pub const MAX_RECONNECT_ATTEMPTS: u32 = 5;
    
    /// Heartbeat interval (in seconds)
    pub const HEARTBEAT_INTERVAL: f32 = 5.0;
    
    /// Get the connection timeout (can be overridden with SPACETIME_CONNECTION_TIMEOUT)
    pub fn get_connection_timeout() -> f32 {
        get_config("SPACETIME_CONNECTION_TIMEOUT", DEFAULT_TIMEOUT)
    }
    
    /// Get the max reconnect attempts (can be overridden with SPACETIME_MAX_RECONNECT_ATTEMPTS)
    pub fn get_max_reconnect_attempts() -> u32 {
        get_config("SPACETIME_MAX_RECONNECT_ATTEMPTS", MAX_RECONNECT_ATTEMPTS)
    }
    
    /// Get the heartbeat interval (can be overridden with SPACETIME_HEARTBEAT_INTERVAL)
    pub fn get_heartbeat_interval() -> f32 {
        get_config("SPACETIME_HEARTBEAT_INTERVAL", HEARTBEAT_INTERVAL)
    }
} 