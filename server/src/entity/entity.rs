// src/entity/entity.rs

use spacetimedb::SpacetimeType;

#[derive(SpacetimeType)]
#[derive(Clone, Copy, Debug)]
pub struct StdbPosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(SpacetimeType)]
#[derive(Clone, Copy, Debug)]
pub struct StdbRotation {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[derive(SpacetimeType)]
#[derive(Clone, Copy, Debug)]
pub struct StdbTransform {
    pub position: StdbPosition,
    pub rotation: StdbRotation,
}
