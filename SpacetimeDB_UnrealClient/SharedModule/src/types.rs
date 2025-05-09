//! # Common Types
//!
//! Common type definitions used across both client and server modules.

use serde::{Serialize, Deserialize};

/// Status code for operation results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatusCode {
    /// Operation succeeded
    Success = 0,
    /// Generic failure
    Failure = 1,
    /// Invalid input parameters
    InvalidInput = 2,
    /// Not authorized to perform operation
    NotAuthorized = 3,
    /// Target not found
    NotFound = 4,
    /// Network error
    NetworkError = 5,
    /// Object already exists
    AlreadyExists = 6,
    /// Operation timed out
    Timeout = 7,
    /// Server is busy
    ServerBusy = 8,
}

/// Result type with string error message
pub type StdbResult<T> = Result<T, String>;

/// Vector3 representation (matches Unreal's FVector)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }
    
    pub fn one() -> Self {
        Self { x: 1.0, y: 1.0, z: 1.0 }
    }
}

/// Rotator representation (matches Unreal's FRotator)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rotator {
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
}

impl Rotator {
    pub fn new(pitch: f32, yaw: f32, roll: f32) -> Self {
        Self { pitch, yaw, roll }
    }
    
    pub fn zero() -> Self {
        Self { pitch: 0.0, yaw: 0.0, roll: 0.0 }
    }
}

/// Quaternion representation (matches Unreal's FQuat)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quat {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }
    
    pub fn identity() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0, w: 1.0 }
    }
}

/// Transform representation (matches Unreal's FTransform)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub location: Vector3,
    pub rotation: Quat,
    pub scale: Vector3,
}

impl Transform {
    pub fn new(location: Vector3, rotation: Quat, scale: Vector3) -> Self {
        Self { location, rotation, scale }
    }
    
    pub fn identity() -> Self {
        Self {
            location: Vector3::zero(),
            rotation: Quat::identity(),
            scale: Vector3::one(),
        }
    }
}

/// Color representation (matches Unreal's FColor)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
    
    pub fn white() -> Self {
        Self { r: 255, g: 255, b: 255, a: 255 }
    }
    
    pub fn black() -> Self {
        Self { r: 0, g: 0, b: 0, a: 255 }
    }
} 