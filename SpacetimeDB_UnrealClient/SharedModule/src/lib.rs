//! # SharedModule
//!
//! Shared types and utilities used by both client and server modules in the
//! SpacetimeDB Unreal integration. This module contains common data structures,
//! type definitions, and serialization formats to ensure consistency across
//! the client-server boundary.

// Export module structure
pub mod types;
pub mod property;
pub mod object;
pub mod constants;

// Re-export commonly used items for convenience
pub use types::*;
pub use property::{PropertyType, PropertyValue};
pub use object::ObjectId; 