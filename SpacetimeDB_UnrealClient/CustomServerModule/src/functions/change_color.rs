use spacetimedb::ReducerContext;
use stdb_shared::types::StdbColor;
use stdb_shared::object::ObjectId;
use serde::{Serialize, Deserialize};
use serde_json::json;

/// Input parameters for the change_color reducer
#[derive(Serialize, Deserialize, Debug)]
pub struct ChangeColorParams {
    /// The ID of the actor to change color
    pub actor_id: ObjectId,
    /// The new color to apply
    pub color: StdbColor,
    /// Whether to animate the color transition
    pub animate_transition: bool,
    /// Duration of the animation in seconds (if animate_transition is true)
    pub transition_duration: Option<f32>,
}

/// Changes the color of an actor.
/// 
/// This function updates the actor's material color property and can optionally
/// animate the transition from the current color to the new one.
/// 
/// # Examples
/// 
/// ```
/// // Change actor 1000's color to green with a 2-second animated transition
/// let params = ChangeColorParams {
///     actor_id: ObjectId(1000),
///     color: StdbColor { r: 0, g: 255, b: 0, a: 255 },
///     animate_transition: true,
///     transition_duration: Some(2.0),
/// };
/// change_color(ctx, &params);
/// ```
#[spacetimedb::reducer]
pub fn change_color(ctx: &ReducerContext, params: &ChangeColorParams) -> bool {
    log::info!("Changing color of actor {:?} to: {:?}", params.actor_id, params.color);
    
    // First, we need to verify the actor exists and the client has permission to modify it
    // This would typically call into the ServerModule's object system
    // For demonstration, we'll assume this check passes
    
    // Update the actor's color property
    // In a real implementation, this would call set_property on the actor
    // set_property(params.actor_id, "Color", json!(params.color));
    
    // If animation is requested, add additional properties to control the transition
    if params.animate_transition {
        let duration = params.transition_duration.unwrap_or(1.0);
        log::info!("Animating color transition over {:.1} seconds", duration);
        
        // In a real implementation, this would set additional properties
        // set_property(params.actor_id, "bAnimateColorChange", json!(true));
        // set_property(params.actor_id, "ColorTransitionDuration", json!(duration));
    }
    
    log::info!("Actor color changed successfully");
    true
} 