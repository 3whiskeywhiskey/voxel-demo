// src/coords.rs

#[cfg(feature = "server")]
use spacetimedb::SpacetimeType;

/// Chunk indices on the XZ plane.
#[cfg_attr(feature = "server", derive(SpacetimeType))]
#[derive(Clone, Copy, Debug)]
pub struct ChunkCoords {
    pub x: i32,
    pub z: i32,
}

/// A 3D vector.
#[cfg_attr(feature = "server", derive(SpacetimeType))]
#[derive(Clone, Copy, Debug)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// A density update at a specific index.
#[cfg_attr(feature = "server", derive(SpacetimeType))]
#[derive(Clone, Copy, Debug)]
pub struct DensityDelta {
    pub index: u32,
    pub value: f32,
}

/// Which material (grass, dirt, stone…) a vertex/face belongs to.
pub type MaterialId = u8;

/// How many voxels per edge of a chunk.
pub const CHUNK_SIZE: usize = 32;