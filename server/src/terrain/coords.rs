// src/coords.rs

use spacetimedb::SpacetimeType;

/// Chunk indices on the XZ plane.
#[derive(SpacetimeType)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct XZCoords {
    pub x: i32,
    pub z: i32,
}

impl XZCoords {
    pub fn to_world_pos(&self, local_x: i32, local_z: i32) -> Vec3 {
        Vec3 {
            x: (self.x * CHUNK_SIZE as i32 + local_x) as f32,
            y: 0.0,
            z: (self.z * CHUNK_SIZE as i32 + local_z as i32) as f32,
        }
    }
}


/// A 3D vector.
#[derive(SpacetimeType)]
#[derive(Clone, Copy, Debug)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let length = self.length();
        Self { x: self.x / length, y: self.y / length, z: self.z / length }
    }
}

/// A density update at a specific index.
#[derive(SpacetimeType)]
#[derive(Clone, Copy, Debug)]
pub struct DensityDelta {
    pub index: u32,
    pub value: f32,
}

/// Which material (grass, dirt, stone…) a vertex/face belongs to.
pub type MaterialId = u32;

/// How many voxels per edge of a chunk.
pub const CHUNK_SIZE: i32 = 32;
pub const SECTION_SIZE: i32 = 32;