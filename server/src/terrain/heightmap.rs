// src/heightmap.rs

use spacetimedb::{table, reducer, ReducerContext, Table};

use crate::terrain::coords::ChunkCoords;

#[table(name = heightmap_chunk, public)]
#[derive(Clone, Debug)]
pub struct HeightmapChunk {
    #[primary_key]
    pub coord: ChunkCoords,
    pub heights: Vec<f32>,
}

#[reducer]
pub fn on_heightmap_generated(ctx: &ReducerContext, chunk: HeightmapChunk) -> Result<(), String> {
    ctx.db.heightmap_chunk().insert(chunk);
    Ok(())
}