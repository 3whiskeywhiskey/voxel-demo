// src/heightmap.rs

#[cfg(feature = "server")]
use spacetimedb::{table, reducer, ReducerContext, Table};

use crate::coords::ChunkCoords;

#[cfg_attr(feature = "server", table(name = heightmap_chunk))]
#[derive(Clone, Debug)]
pub struct HeightmapChunk {
    #[cfg_attr(feature = "server", primary_key)]
    pub coord: ChunkCoords,
    pub heights: Vec<f32>,
}

#[cfg(feature = "server")]
#[reducer]
pub fn on_heightmap_generated(ctx: &ReducerContext, chunk: HeightmapChunk) -> Result<(), String> {
    ctx.db.heightmap_chunk().insert(chunk);
    Ok(())
}