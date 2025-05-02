// src/heightmap.rs

use spacetimedb::{table, reducer, ReducerContext, Table};

use crate::terrain::coords::ChunkCoords;

#[table(name = heightmap_chunk, index(name = idx_chunk_xz, btree(columns = [chunk_x, chunk_z])), public)]
#[derive(Clone, Debug)]
pub struct HeightmapChunk {
    #[primary_key]
    pub coord: ChunkCoords,

    pub chunk_x: i32,
    pub chunk_z: i32,

    pub heights: Vec<f32>,
}

#[reducer]
pub fn on_heightmap_generated(
    ctx: &ReducerContext,
    coord: ChunkCoords,
    heights: Vec<f32>,
) -> Result<(), String> {
    let table = ctx.db.heightmap_chunk();

    let chunk = HeightmapChunk {
        coord,
        chunk_x: coord.x,
        chunk_z: coord.z,
        heights,
    };

    match table.try_insert(chunk.clone()) {
        // Insert succeeded (new chunk)
        Ok(_inserted_chunk) => Ok(()),

        Err(_) => {
            // Chunk already existed—overwrite via update (panics on failure)
            table.coord().update(chunk);
            Ok(())
        }
    }
}