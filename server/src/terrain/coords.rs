// src/coords.rs

use spacetimedb::SpacetimeType;

/// Chunk indices on the XZ plane.
#[derive(SpacetimeType)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ChunkCoords {
    pub x: i32,
    pub z: i32,
}

impl ChunkCoords {
    pub fn to_world_pos(&self, local_x: usize, local_z: usize) -> Vec3 {
        Vec3 {
            x: (self.x * CHUNK_SIZE as i32 + local_x as i32) as f32,
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

/// A density update at a specific index.
#[derive(SpacetimeType)]
#[derive(Clone, Copy, Debug)]
pub struct DensityDelta {
    pub index: u32,
    pub value: f32,
}

/// Which material (grass, dirt, stone…) a vertex/face belongs to.
pub type MaterialId = u8;

/// How many voxels per edge of a chunk.
pub const CHUNK_SIZE: usize = 32;