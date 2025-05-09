//! # UObject Class System
//!
//! Handles registration and management of UClass definitions, providing the 
//! reflection capabilities that are core to Unreal's object model.

use spacetimedb::ReducerContext;
use crate::object::{ObjectClass, ClassProperty};
use crate::property::PropertyType;

/// Initializes the UObject class system with all Unreal Engine classes
pub fn initialize_object_classes(ctx: &ReducerContext) {
    log::info!("Initializing UObject class system");
    
    // Register all classes from the code generator
    // This includes both core engine classes (UObject, AActor, etc.) and project-specific classes
    crate::generated::register_all_classes(ctx);
    
    // Register all properties from the code generator 
    // This includes properties for both core engine classes and project-specific classes
    crate::generated::register_all_properties(ctx);
    
    log::info!("UObject class system initialization complete");
}

/// Registers a custom class with the specified parameters
///
/// Returns the assigned class_id for the new class
pub fn register_custom_class(
    ctx: &ReducerContext,
    class_name: String,
    class_path: String,
    parent_class_id: u32,
    replicates: bool,
    is_actor: bool,
    is_component: bool,
) -> u32 {
    log::info!("Registering custom class: {} (parent: {})", class_name, parent_class_id);
    
    // Generate a new class ID
    let class_id = generate_class_id(ctx);
    
    // Create the class object
    let new_class = ObjectClass {
        class_id,
        class_name: class_name.clone(),
        class_path,
        parent_class_id,
        replicates,
        is_actor,
        is_component,
    };
    
    // Register the class
    ctx.db.object_class().insert(new_class);
    
    log::info!("Registered custom class {} with ID {}", class_name, class_id);
    
    class_id
}

/// Registers a property for a class
pub fn register_class_property(
    ctx: &ReducerContext,
    class_id: u32,
    property_name: String,
    property_type: PropertyType,
    replicated: bool,
    readonly: bool,
) -> bool {
    // Verify the class exists
    if ctx.db.object_class().filter_by_class_id(&class_id).first().is_none() {
        log::error!("Attempted to register property for non-existent class: {}", class_id);
        return false;
    }
    
    // Create the property
    let property = ClassProperty {
        class_id,
        property_name: property_name.clone(),
        property_type,
        replicated,
        readonly,
    };
    
    // Register the property
    ctx.db.class_property().insert(property);
    
    log::info!("Registered property {} for class {}", property_name, class_id);
    
    true
}

/// Generates a unique class ID
fn generate_class_id(ctx: &ReducerContext) -> u32 {
    // Find the highest existing class ID
    let max_id = ctx.db.object_class()
        .iter()
        .map(|c| c.class_id)
        .max()
        .unwrap_or(99); // Start at 100 if no classes exist
    
    // Return the next ID
    max_id + 1
} 