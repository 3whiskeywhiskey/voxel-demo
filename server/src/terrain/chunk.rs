// src/chunk.rs

use spacetimedb::{table, reducer, ReducerContext, Table};

use crate::terrain::coords::ChunkCoords;
use crate::terrain::generator::HeightmapGenerator;
use once_cell::sync::OnceCell;

static HEIGHTMAP_GENERATOR: OnceCell<HeightmapGenerator> = OnceCell::new();


#[table(name = chunk, index(name = idx_chunk_xz, btree(columns = [chunk_x, chunk_z])), public)]
#[derive(Clone, Debug)]
pub struct Chunk {
    #[primary_key]
    pub coord: ChunkCoords,

    pub chunk_x: i32,
    pub chunk_z: i32,

    pub heights: Vec<f32>,
}

#[reducer]
pub fn on_chunk_requested(
    ctx: &ReducerContext,
    coord: ChunkCoords,
) -> Result<(), String> {
    let table = ctx.db.chunk();

    let heights =  HEIGHTMAP_GENERATOR
        .get_or_init(|| HeightmapGenerator::new(42))
        .generate_chunk(coord);

    let chunk = Chunk {
        coord,
        chunk_x: coord.x,
        chunk_z: coord.z,
        heights,
    };

    match table.try_insert(chunk.clone()) {
        // Insert succeeded (new chunk)
        Ok(_) => Ok(()),
        Err(_) => {
            // Chunk already existed—overwrite via update (panics on failure)
            table.coord().update(chunk);
            Ok(())
        }
    }
}


