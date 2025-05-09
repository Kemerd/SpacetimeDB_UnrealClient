//! # Actor Lifecycle
//!
//! Handles actor lifecycle operations like destruction and state transitions.

use spacetimedb::ReducerContext;
use crate::actor::ActorId;
use crate::object::{ObjectInstance, ObjectLifecycleState};

/// Marks an actor for destruction
#[spacetimedb::reducer]
pub fn destroy_actor(ctx: &ReducerContext, actor_id: ActorId) -> bool {
    // Check if actor exists
    let actor = match ctx.db.object_instance().filter_by_object_id(&actor_id).first() {
        Some(actor) => {
            // Verify it's actually an actor
            if !actor.is_actor {
                log::warn!("Attempted to destroy object {} as an actor, but it's not an actor", actor_id);
                return false;
            }
            actor
        },
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
    updated_actor.state = ObjectLifecycleState::PendingDestroy;
    // Record the destruction timestamp
    updated_actor.destroyed_at = Some(ctx.timestamp);
    ctx.db.object_instance().update(&updated_actor);
    
    log::info!("Actor {} marked for destruction by {:?}", actor_id, ctx.sender);
    
    // Notify relevant systems about pending destruction
    notify_actor_pending_destroy(ctx, actor_id);
    
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
    if let Some(mut actor) = ctx.db.object_instance().filter_by_object_id(&actor_id).first() {
        // Verify it's actually an actor
        if !actor.is_actor {
            log::warn!("Attempted to force destroy object {} as an actor, but it's not an actor", actor_id);
            return false;
        }
        
        // Mark as destroyed
        actor.state = ObjectLifecycleState::Destroyed;
        actor.destroyed_at = Some(ctx.timestamp);
        ctx.db.object_instance().update(&actor);
        
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
    let actor = match ctx.db.object_instance().filter_by_object_id(&actor_id).first() {
        Some(actor) => {
            // Verify it's actually an actor
            if !actor.is_actor {
                log::warn!("Attempted to modify visibility of object {} as an actor, but it's not an actor", actor_id);
                return false;
            }
            actor
        },
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
    ctx.db.object_instance().update(&updated_actor);
    
    log::info!("Actor {} visibility set to hidden={} by {:?}", actor_id, hidden, ctx.sender);
    true
}

/// Runs periodically to clean up destroyed actors and handle pending operations
#[spacetimedb::reducer(scheduled = "1000ms")]
pub fn cleanup_destroyed_actors(ctx: &ReducerContext) {
    log::debug!("Running actor cleanup");
    
    // Find all actors in PendingDestroy state
    let pending_destroy: Vec<_> = ctx.db.object_instance()
        .iter()
        .filter(|obj| obj.is_actor && obj.state == ObjectLifecycleState::PendingDestroy)
        .collect();
    
    for actor in pending_destroy {
        log::info!("Cleaning up pending destroy actor: {}", actor.object_id);
        
        // Update state to Destroyed
        let mut updated_actor = actor.clone();
        updated_actor.state = ObjectLifecycleState::Destroyed;
        
        // Ensure destruction timestamp is set if missing
        if updated_actor.destroyed_at.is_none() {
            updated_actor.destroyed_at = Some(ctx.timestamp);
        }
        
        ctx.db.object_instance().update(&updated_actor);
        
        // Clean up any temporary resources
        cleanup_actor_resources(ctx, actor.object_id);
    }
    
    // Set a reasonable time delay for fully removing destroyed actors (10 seconds)
    let destruction_delay_ms: u64 = 10000;
    
    // Find actors that have been in Destroyed state for long enough to be deleted
    let destroyed: Vec<_> = ctx.db.object_instance()
        .iter()
        .filter(|obj| obj.is_actor && obj.state == ObjectLifecycleState::Destroyed)
        .filter(|actor| {
            if let Some(destroyed_at) = actor.destroyed_at {
                // Use the proper destruction timestamp
                return ctx.timestamp - destroyed_at > destruction_delay_ms;
            }
            false
        })
        .collect();
    
    for actor in destroyed {
        log::info!("Fully deleting destroyed actor: {}", actor.object_id);
        
        // Delete the actor and all related data
        
        // Delete components
        for component in ctx.db.object_component()
            .iter()
            .filter(|c| c.owner_object_id == actor.object_id) {
            ctx.db.object_component().delete_by_component_id(&component.component_id);
        }
        
        // Delete properties
        ctx.db.object_property().delete_by_object_id(&actor.object_id);
        
        // Delete transform
        ctx.db.object_transform().delete_by_object_id(&actor.object_id);
        
        // Delete any relevancy data
        remove_from_relevancy_system(ctx, actor.object_id);
        
        // Delete any RPC handlers
        deregister_rpc_handlers(ctx, actor.object_id);
        
        // Finally delete the actor instance
        ctx.db.object_instance().delete_by_object_id(&actor.object_id);
    }
}

/// Notifies relevant systems about a pending actor destruction
fn notify_actor_pending_destroy(ctx: &ReducerContext, actor_id: ActorId) {
    log::debug!("Notifying systems of pending destroy for actor {}", actor_id);
    
    // Notify relevancy system to prepare clients for this actor's removal
    crate::relevancy::notify_actor_pending_destroy(ctx, actor_id);
    
    // Cancel any pending actions or timers for this actor
    crate::action::cancel_pending_actions(ctx, actor_id);
    
    // Notify any attached components
    for component in ctx.db.object_component()
        .iter()
        .filter(|c| c.owner_object_id == actor_id) {
        // Signal components about destruction
        crate::component::notify_pending_destroy(ctx, component.component_id);
    }
}

/// Cleans up resources associated with an actor that's being destroyed
fn cleanup_actor_resources(ctx: &ReducerContext, actor_id: ActorId) {
    log::debug!("Cleaning up resources for actor {}", actor_id);
    
    // Remove from any gameplay systems
    crate::gameplay::remove_actor_from_gameplay(ctx, actor_id);
    
    // Remove from any active zones
    crate::zone::remove_actor_from_all_zones(ctx, actor_id);
    
    // Release any reserved resources
    crate::resource::release_actor_resources(ctx, actor_id);
}

/// Removes actor from the relevancy system during final cleanup
fn remove_from_relevancy_system(ctx: &ReducerContext, actor_id: ActorId) {
    log::debug!("Removing actor {} from relevancy system", actor_id);
    crate::relevancy::remove_actor(ctx, actor_id);
}

/// Deregisters any RPC handlers associated with the actor
fn deregister_rpc_handlers(ctx: &ReducerContext, actor_id: ActorId) {
    log::debug!("Deregistering RPC handlers for actor {}", actor_id);
    crate::rpc::deregister_handlers_for_actor(ctx, actor_id);
}

/// Checks if a client can modify the given actor
fn can_modify_actor(ctx: &ReducerContext, actor: &ObjectInstance) -> bool {
    // Verify it's an actor
    if !actor.is_actor {
        return false;
    }
    
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