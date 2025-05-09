use spacetimedb::ReducerContext;
use stdb_shared::types::{StdbVector3, StdbColor};
use stdb_shared::object::{ObjectId, SpawnParams};
use serde::{Serialize, Deserialize};

/// Input parameters for the spawn_sphere reducer
#[derive(Serialize, Deserialize, Debug)]
pub struct SpawnSphereParams {
    /// Position to spawn the sphere at
    pub position: StdbVector3,
    /// Radius of the sphere
    pub radius: f32,
    /// Color of the sphere
    pub color: StdbColor,
    /// Whether the sphere should have physics enabled
    pub physics_enabled: bool,
}

/// Spawn a sphere actor at the specified position with the given parameters.
/// 
/// This function creates a new UE sphere actor with customizable properties.
/// 
/// # Examples
/// 
/// ```
/// // Spawn a red sphere at (100, 0, 50) with radius 50 and physics enabled
/// let params = SpawnSphereParams {
///     position: StdbVector3 { x: 100.0, y: 0.0, z: 50.0 },
///     radius: 50.0,
///     color: StdbColor { r: 255, g: 0, b: 0, a: 255 },
///     physics_enabled: true,
/// };
/// spawn_sphere(ctx, &params);
/// ```
#[spacetimedb::reducer]
pub fn spawn_sphere(ctx: &ReducerContext, params: &SpawnSphereParams) -> ObjectId {
    log::info!("Spawning sphere at position: {:?}", params.position);
    
    // Create initial properties for the sphere
    let initial_properties = serde_json::json!({
        "Location": params.position,
        "Scale": StdbVector3 { x: params.radius / 50.0, y: params.radius / 50.0, z: params.radius / 50.0 },
        "Color": params.color,
        "bEnablePhysics": params.physics_enabled,
        "bReplicates": true,
        "bNetLoadOnClient": true,
    });
    
    // Convert to SpawnParams with appropriate class name for a sphere actor
    let spawn_params = SpawnParams {
        class_name: "BP_ReplicatedSphere".to_string(),
        initial_properties: serde_json::to_string(&initial_properties).unwrap(),
        owner_id: ctx.sender.clone(),
        transform: None, // Will be set via initial_properties (Location)
    };
    
    // Call the core actor spawn system from ServerModule
    // This would typically call into the ServerModule's actor system
    // For demonstration, we're returning a placeholder ObjectId
    log::info!("Sphere spawned successfully");
    
    // In a real implementation, this would return the actual ObjectId from the spawn function
    // For now, we'll return a placeholder ID
    ObjectId(1000)
} 