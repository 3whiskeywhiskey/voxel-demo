// src/terrain/chunk.rs

use spacetimedb::{table, reducer, ReducerContext, Table};

use crate::entity::mesh::mesh;
use crate::terrain::coords::ChunkCoords;
use crate::terrain::generator::{HeightmapGenerator, MeshGenerator, PaddedHeightmap}  ;
use once_cell::sync::OnceCell;

static HEIGHTMAP_GENERATOR: OnceCell<HeightmapGenerator> = OnceCell::new();
static MESH_GENERATOR: OnceCell<MeshGenerator> = OnceCell::new();

#[table(
    name = chunk, 
    index(name = idx_chunk_xz, btree(columns = [chunk_x, chunk_z])), 
    index(name = idx_mesh_id, btree(columns = [mesh_id])),
    public
)]
#[derive(Clone, Debug)]
pub struct Chunk {
    #[primary_key]
    pub coord: ChunkCoords,

    pub chunk_x: i32,
    pub chunk_z: i32,

    pub heights: Vec<f32>,
    pub mesh_id: u64,
}

#[reducer]
pub fn on_chunk_requested(
    ctx: &ReducerContext,
    coord: ChunkCoords,
) -> Result<(), String> {
    let chunk_table = ctx.db.chunk();

    let heights =  HEIGHTMAP_GENERATOR
        .get_or_init(|| HeightmapGenerator::new(42))
        .generate_chunk(coord);

    let padded_heightmap =  HEIGHTMAP_GENERATOR
        .get_or_init(|| HeightmapGenerator::new(42))
        .generate_padded_heightmap(coord);

    let mesh = MESH_GENERATOR
        .get_or_init(|| MeshGenerator::new())
        .generate_mesh(coord, padded_heightmap);

    let mesh_table = ctx.db.mesh();
    let mesh_result = mesh_table.try_insert(mesh.clone());

    let chunk = Chunk {
        coord,
        chunk_x: coord.x,
        chunk_z: coord.z,
        heights,
        mesh_id: mesh_result.unwrap().id,
    };

    match chunk_table.try_insert(chunk.clone()) {
        // Insert succeeded (new chunk)
        Ok(_) => Ok(()),
        Err(_) => {
            // Chunk already existed—overwrite via update (panics on failure)
            // chunk_table.update(chunk);
            Ok(())
        }
    }
}


