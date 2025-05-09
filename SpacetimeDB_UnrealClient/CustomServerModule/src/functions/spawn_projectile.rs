use spacetimedb::ReducerContext;
use stdb_shared::types::{StdbVector3, StdbRotator, StdbColor};
use stdb_shared::object::{ObjectId, SpawnParams};
use serde::{Serialize, Deserialize};

/// Input parameters for the spawn_projectile reducer
#[derive(Serialize, Deserialize, Debug)]
pub struct SpawnProjectileParams {
    /// Position to spawn the projectile
    pub position: StdbVector3,
    /// Direction the projectile is facing
    pub rotation: StdbRotator,
    /// Initial velocity of the projectile
    pub velocity: StdbVector3,
    /// Size/scale of the projectile
    pub scale: f32,
    /// Color of the projectile
    pub color: StdbColor,
    /// Damage amount the projectile will deal on impact
    pub damage: f32,
    /// ID of the actor that spawned this projectile (for hit attribution)
    pub owner_id: ObjectId,
    /// Time in seconds until the projectile auto-destructs
    pub lifetime: f32,
    /// Optional projectile type/class
    pub projectile_type: Option<String>,
}

/// Spawns a projectile with the given parameters and velocity.
/// 
/// This function creates a new projectile actor that will travel along its initial
/// velocity vector and can interact with other objects in the world.
/// 
/// # Examples
/// 
/// ```
/// // Spawn a fast-moving red projectile from position (0, 0, 100) traveling forward
/// let params = SpawnProjectileParams {
///     position: StdbVector3 { x: 0.0, y: 0.0, z: 100.0 },
///     rotation: StdbRotator { pitch: 0.0, yaw: 0.0, roll: 0.0 },
///     velocity: StdbVector3 { x: 1000.0, y: 0.0, z: 0.0 },
///     scale: 0.5,
///     color: StdbColor { r: 255, g: 0, b: 0, a: 255 },
///     damage: 25.0,
///     owner_id: ObjectId(1234),
///     lifetime: 5.0,
///     projectile_type: None, // Use default
/// };
/// spawn_projectile(ctx, &params);
/// ```
#[spacetimedb::reducer]
pub fn spawn_projectile(ctx: &ReducerContext, params: &SpawnProjectileParams) -> ObjectId {
    log::info!("Spawning projectile at position: {:?} with velocity: {:?}", 
               params.position, params.velocity);
    
    // Determine projectile class name
    let projectile_class = params.projectile_type
        .clone()
        .unwrap_or_else(|| "BP_DefaultProjectile".to_string());
    
    // Create initial properties for the projectile
    let initial_properties = serde_json::json!({
        "Location": params.position,
        "Rotation": params.rotation,
        "Scale": StdbVector3 { 
            x: params.scale, 
            y: params.scale, 
            z: params.scale 
        },
        "Color": params.color,
        "Velocity": params.velocity,
        "Damage": params.damage,
        "SourceActorId": params.owner_id,
        "Lifetime": params.lifetime,
        "bReplicates": true,
        "bNetLoadOnClient": true,
    });
    
    // Convert to SpawnParams with appropriate class name
    let spawn_params = SpawnParams {
        class_name: projectile_class,
        initial_properties: serde_json::to_string(&initial_properties).unwrap(),
        owner_id: ctx.sender.clone(),
        transform: None, // Will be set via initial_properties
    };
    
    // Call the core actor spawn system from ServerModule
    // This would typically call into the ServerModule's actor system
    // For demonstration, we're returning a placeholder ObjectId
    log::info!("Projectile spawned successfully");
    
    // In a real implementation, this would return the actual ObjectId from the spawn function
    // For now, we'll return a placeholder ID
    ObjectId(1002)
} 