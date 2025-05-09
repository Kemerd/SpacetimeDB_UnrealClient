//! # Actor Lifecycle
//!
//! Handles actor lifecycle operations like destruction and state transitions.

use spacetimedb::ReducerContext;
use crate::actor::{ActorId, ActorLifecycleState};

/// Marks an actor for destruction
#[spacetimedb::reducer]
pub fn destroy_actor(ctx: &ReducerContext, actor_id: ActorId) -> bool {
    // Check if actor exists
    let actor = match ctx.db.actor_info().filter_by_actor_id(&actor_id).first() {
        Some(actor) => actor,
        None => {
            log::warn!("Attempted to destroy non-existent actor: {}", actor_id);
            return false;
        }
    };
    
    // Check permission to destroy this actor
    if !can_modify_actor(ctx, &actor) {
        log::warn!("Client {:?} doesn't have permission to destroy actor {}", ctx.sender, actor_id);
        return false;
    }
    
    // Mark as pending destroy
    let mut updated_actor = actor.clone();
    updated_actor.state = ActorLifecycleState::PendingDestroy;
    ctx.db.actor_info().update(&updated_actor);
    
    log::info!("Actor {} marked for destruction by {:?}", actor_id, ctx.sender);
    
    // Schedule cleanup for the next tick
    // In a real implementation, you'd have a system to clean up destroyed actors
    
    true
}

/// Immediately destroys an actor (admin/system only)
#[spacetimedb::reducer]
pub fn force_destroy_actor(ctx: &ReducerContext, actor_id: ActorId) -> bool {
    // Only admins or system can force destroy
    if !crate::connection::auth::is_admin(ctx) {
        log::warn!("Non-admin client attempted to force destroy actor: {:?}", ctx.sender);
        return false;
    }
    
    // Check if actor exists
    if let Some(mut actor) = ctx.db.actor_info().filter_by_actor_id(&actor_id).first() {
        // Mark as destroyed
        actor.state = ActorLifecycleState::Destroyed;
        ctx.db.actor_info().update(&actor);
        
        log::info!("Actor {} force destroyed by {:?}", actor_id, ctx.sender);
        return true;
    } else {
        log::warn!("Attempted to force destroy non-existent actor: {}", actor_id);
        return false;
    }
}

/// Hides or shows an actor (visual only, doesn't affect simulation)
#[spacetimedb::reducer]
pub fn set_actor_hidden(ctx: &ReducerContext, actor_id: ActorId, hidden: bool) -> bool {
    // Find the actor
    let actor = match ctx.db.actor_info().filter_by_actor_id(&actor_id).first() {
        Some(actor) => actor,
        None => {
            log::warn!("Attempted to modify visibility of non-existent actor: {}", actor_id);
            return false;
        }
    };
    
    // Check permission to modify this actor
    if !can_modify_actor(ctx, &actor) {
        log::warn!("Client {:?} doesn't have permission to modify actor {}", ctx.sender, actor_id);
        return false;
    }
    
    // Update the hidden state
    let mut updated_actor = actor.clone();
    updated_actor.hidden = hidden;
    ctx.db.actor_info().update(&updated_actor);
    
    log::info!("Actor {} visibility set to hidden={} by {:?}", actor_id, hidden, ctx.sender);
    true
}

/// Runs periodically to clean up destroyed actors and handle pending operations
#[spacetimedb::reducer(scheduled = "1000ms")]
pub fn cleanup_destroyed_actors(ctx: &ReducerContext) {
    log::debug!("Running actor cleanup");
    
    // Find all actors in PendingDestroy state
    let pending_destroy: Vec<_> = ctx.db.actor_info()
        .iter()
        .filter(|actor| actor.state == ActorLifecycleState::PendingDestroy)
        .collect();
    
    for actor in pending_destroy {
        log::info!("Cleaning up pending destroy actor: {}", actor.actor_id);
        
        // Update state to Destroyed
        let mut updated_actor = actor.clone();
        updated_actor.state = ActorLifecycleState::Destroyed;
        ctx.db.actor_info().update(&updated_actor);
        
        // In a real implementation, you might clean up related data,
        // or delay deletion further to ensure clients have time to process the destruction
    }
    
    // Find actors that have been in Destroyed state for a while and delete them
    // (In a real implementation, you'd check timestamps)
    let destroyed: Vec<_> = ctx.db.actor_info()
        .iter()
        .filter(|actor| actor.state == ActorLifecycleState::Destroyed)
        .filter(|actor| ctx.timestamp - actor.created_at > 60000) // 60 seconds for example
        .collect();
    
    for actor in destroyed {
        log::info!("Fully deleting destroyed actor: {}", actor.actor_id);
        
        // Delete the actor and all related data
        
        // Delete components
        for component in ctx.db.actor_component()
            .iter()
            .filter(|c| c.owner_actor_id == actor.actor_id) {
            ctx.db.actor_component().delete_by_component_id(&component.component_id);
        }
        
        // Delete properties
        ctx.db.actor_property().delete_by_actor_id(&actor.actor_id);
        
        // Delete transform
        ctx.db.actor_transform().delete_by_actor_id(&actor.actor_id);
        
        // Finally delete the actor info
        ctx.db.actor_info().delete_by_actor_id(&actor.actor_id);
    }
}

/// Checks if a client can modify the given actor
fn can_modify_actor(ctx: &ReducerContext, actor: &crate::actor::ActorInfo) -> bool {
    // If there's no client (system call), always allow
    if ctx.sender.is_none() {
        return true;
    }
    
    // Admins can modify any actor
    if crate::connection::auth::is_admin(ctx) {
        return true;
    }
    
    // Owners can modify their own actors
    if let Some(sender) = ctx.sender {
        if let Some(owner) = actor.owner_identity {
            return sender == owner;
        }
    }
    
    // Default to not allowed
    false
} 