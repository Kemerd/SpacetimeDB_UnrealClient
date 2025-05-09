//! # Custom Game Server Module
//! 
//! This module provides custom game-specific functionality built on top of
//! the core UnrealReplication server module. It contains example reducers and
//! functions specific to your game.

use spacetimedb::ReducerContext;

// Include all function modules from the functions directory
mod functions;

// Re-export all game-specific functions for easier access
pub use functions::*;

/// Initialize the custom game module
#[spacetimedb::reducer(init)]
pub fn init_custom_module(ctx: &ReducerContext) {
    log::info!("CustomServerModule initialized");
    
    // Any custom game initialization can go here
    log::info!("Custom game systems ready");
} 