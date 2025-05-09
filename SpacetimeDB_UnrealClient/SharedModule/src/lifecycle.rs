//! # Object Lifecycle Types
//!
//! Shared types for object and actor lifecycle management.
//! This includes state enums for tracking the lifecycle stages of objects and actors.

use serde::{Serialize, Deserialize};

/// The current state of an object in its lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectLifecycleState {
    /// Object is being created but not yet fully initialized
    Initializing,
    
    /// Object is active and valid
    Active,
    
    /// Object is being cleaned up for destruction
    PendingKill,
    
    /// Object has been destroyed but is kept in the database for delayed cleanup
    Destroyed,
}

/// The current state of an actor in its lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActorLifecycleState {
    /// Actor is being created but not yet fully initialized
    Spawning,
    
    /// Actor is active in the world
    Active,
    
    /// Actor is being destroyed
    PendingDestroy,
    
    /// Actor has been destroyed but is kept in the database for delayed cleanup
    Destroyed,
}

/// Types of object/actor creation events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CreationEventType {
    /// Object was just created
    Created,
    
    /// Object was destroyed
    Destroyed,
    
    /// Object was updated
    Updated,
}

/// Types of relevant objects for notification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelevantObjectType {
    /// Regular UObject
    Object,
    
    /// Actor
    Actor,
    
    /// Component
    Component,
}

/// Visibility flags for network relevancy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VisibilityFlag {
    /// Visible to all clients
    AllClients,
    
    /// Visible only to the owner
    OwnerOnly,
    
    /// Visible to specific clients
    SpecificClients,
    
    /// Not visible to any clients
    Hidden,
} 