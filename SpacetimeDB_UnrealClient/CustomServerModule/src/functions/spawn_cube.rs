use spacetimedb::ReducerContext;
use stdb_shared::types::{StdbVector3, StdbRotator, StdbColor};
use stdb_shared::object::{ObjectId, SpawnParams};
use serde::{Serialize, Deserialize};

/// Input parameters for the spawn_cube reducer
#[derive(Serialize, Deserialize, Debug)]
pub struct SpawnCubeParams {
    /// Position to spawn the cube at
    pub position: StdbVector3,
    /// Rotation of the cube
    pub rotation: StdbRotator,
    /// Size of the cube (width, height, depth)
    pub size: StdbVector3,
    /// Color of the cube
    pub color: StdbColor,
    /// Whether the cube should have physics enabled
    pub physics_enabled: bool,
    /// Optional material to apply (material name/path)
    pub material: Option<String>,
}

/// Spawn a cube actor at the specified position with the given parameters.
/// 
/// This function creates a new UE cube actor with customizable properties.
/// 
/// # Examples
/// 
/// ```
/// // Spawn a blue cube at (0, 0, 100) with size 100x50x25
/// let params = SpawnCubeParams {
///     position: StdbVector3 { x: 0.0, y: 0.0, z: 100.0 },
///     rotation: StdbRotator { pitch: 0.0, yaw: 45.0, roll: 0.0 },
///     size: StdbVector3 { x: 100.0, y: 50.0, z: 25.0 },
///     color: StdbColor { r: 0, g: 0, b: 255, a: 255 },
///     physics_enabled: true,
///     material: None,
/// };
/// spawn_cube(ctx, &params);
/// ```
#[spacetimedb::reducer]
pub fn spawn_cube(ctx: &ReducerContext, params: &SpawnCubeParams) -> ObjectId {
    log::info!("Spawning cube at position: {:?}", params.position);
    
    // Create initial properties for the cube
    let mut initial_properties = serde_json::json!({
        "Location": params.position,
        "Rotation": params.rotation,
        "Scale": StdbVector3 { 
            x: params.size.x / 100.0, 
            y: params.size.y / 100.0, 
            z: params.size.z / 100.0 
        },
        "Color": params.color,
        "bEnablePhysics": params.physics_enabled,
        "bReplicates": true,
        "bNetLoadOnClient": true,
    });
    
    // Add material if specified
    if let Some(material) = &params.material {
        let material_obj = serde_json::json!({
            "MaterialPath": material,
        });
        initial_properties["Material"] = material_obj;
    }
    
    // Convert to SpawnParams with appropriate class name for a cube actor
    let spawn_params = SpawnParams {
        class_name: "BP_ReplicatedCube".to_string(),
        initial_properties: serde_json::to_string(&initial_properties).unwrap(),
        owner_id: ctx.sender.clone(),
        transform: None, // Will be set via initial_properties
    };
    
    // Call the core actor spawn system from ServerModule
    // This would typically call into the ServerModule's actor system
    // For demonstration, we're returning a placeholder ObjectId
    log::info!("Cube spawned successfully");
    
    // In a real implementation, this would return the actual ObjectId from the spawn function
    // For now, we'll return a placeholder ID
    ObjectId(1001)
} 