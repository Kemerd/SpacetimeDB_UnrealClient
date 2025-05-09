//! # Shared Constants
//!
//! Constants used across both client and server modules.

/// Version of the SpacetimeDB Unreal Client
pub const STDB_CLIENT_VERSION: &str = "0.1.0";

/// Maximum supported number of objects
pub const MAX_OBJECTS: usize = 100_000;

/// Replication constants
pub mod replication {
    /// Default replication interval (in seconds)
    pub const DEFAULT_REPLICATION_INTERVAL: f32 = 0.1;

    /// Maximum properties per replication frame
    pub const MAX_PROPERTIES_PER_FRAME: usize = 1000;

    /// Maximum objects per replication frame
    pub const MAX_OBJECTS_PER_FRAME: usize = 100;
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
    /// Default connection timeout (in seconds)
    pub const DEFAULT_TIMEOUT: f32 = 30.0;
    
    /// Maximum number of reconnection attempts
    pub const MAX_RECONNECT_ATTEMPTS: u32 = 5;
    
    /// Heartbeat interval (in seconds)
    pub const HEARTBEAT_INTERVAL: f32 = 5.0;
} 