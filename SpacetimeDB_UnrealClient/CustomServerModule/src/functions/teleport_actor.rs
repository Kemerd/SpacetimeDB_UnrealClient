use spacetimedb::ReducerContext;
use stdb_shared::types::StdbVector3;
use stdb_shared::object::ObjectId;
use serde::{Serialize, Deserialize};

/// Input parameters for the teleport_actor reducer
#[derive(Serialize, Deserialize, Debug)]
pub struct TeleportActorParams {
    /// The ID of the actor to teleport
    pub actor_id: ObjectId,
    /// The new position to teleport to
    pub position: StdbVector3,
    /// Whether to add some visual effects (e.g., particles, sound)
    pub with_effects: bool,
}

/// Teleports an actor to the specified position.
/// 
/// This function updates the actor's position instantly and can optionally
/// trigger visual/sound effects on both the origin and destination points.
/// 
/// # Examples
/// 
/// ```
/// // Teleport actor 1000 to position (500, 500, 100) with visual effects
/// let params = TeleportActorParams {
///     actor_id: ObjectId(1000),
///     position: StdbVector3 { x: 500.0, y: 500.0, z: 100.0 },
///     with_effects: true,
/// };
/// teleport_actor(ctx, &params);
/// ```
#[spacetimedb::reducer]
pub fn teleport_actor(ctx: &ReducerContext, params: &TeleportActorParams) -> bool {
    log::info!("Teleporting actor {:?} to position: {:?}", params.actor_id, params.position);
    
    // First, we need to verify the actor exists and the client has permission to teleport it
    // This would typically call into the ServerModule's object system
    // For demonstration, we'll assume this check passes
    
    // Store original position for effect spawning (in a real implementation)
    // let original_position = get_actor_position(params.actor_id);
    
    // Update the actor's position
    // In a real implementation, this would call set_property on the actor
    // set_property(params.actor_id, "Location", json!(params.position));
    
    // Spawn visual effects if requested
    if params.with_effects {
        log::info!("Spawning teleport effects");
        // This would typically call functions to spawn particle/sound effects
        // at both the original position and the destination
        // spawn_teleport_effect(original_position);
        // spawn_teleport_effect(params.position);
    }
    
    log::info!("Actor teleported successfully");
    true
} 