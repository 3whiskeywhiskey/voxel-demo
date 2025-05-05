// src/terrain/chunk.rs

use spacetimedb::{table, reducer, ReducerContext, Table};

use crate::terrain::coords::XZCoords;
use crate::terrain::generator::{HeightmapGenerator, MeshGenerator};
use once_cell::sync::OnceCell;

static HEIGHTMAP_GENERATOR: OnceCell<HeightmapGenerator> = OnceCell::new();
static MESH_GENERATOR: OnceCell<MeshGenerator> = OnceCell::new();

#[table(
    name = chunk_vertex,
    index(name = idx_grid_xz, btree(columns = [grid_x, grid_z])),
    public
)]
#[derive(Clone, Debug)]
pub struct ChunkVertex {
    // these need a primary key that's unique, and the composite key works for that
    // but i can't query the composite key in a spatial filter, so i need grid_x and grid_z for that
    #[primary_key]
    pub grid: XZCoords,
    pub grid_x: i32,
    pub grid_z: i32,
    pub heightmap: Vec<f32>,
    pub vertices: Vec<f32>,
    pub normals: Vec<f32>,
}

#[table(
    name = chunk_mesh, 
    index(name = idx_grid_xz, btree(columns = [grid_x, grid_z])), 
    public
)]
#[derive(Clone, Debug)]
pub struct ChunkMesh {
    #[primary_key]
    pub grid: XZCoords,
    pub grid_x: i32,
    pub grid_z: i32,

    pub indices: Vec<u32>,
    pub materials: Vec<u32>,
}

#[reducer]
pub fn on_chunk_requested(
    ctx: &ReducerContext,
    coord: XZCoords,
) -> Result<(), String> {
    let chunk_vertex_table = ctx.db.chunk_vertex();
    let chunk_mesh_table = ctx.db.chunk_mesh();

    let neighborhood = chunk_vertex_table.iter()
        .filter(|chunk| {
            chunk.grid_x.abs_diff(coord.x) <= 1 && chunk.grid_z.abs_diff(coord.z) <= 1
        })
        .collect::<Vec<_>>();
    // info!("Neighborhood: {:?}", neighborhood.iter().map(|chunk| chunk.coord).collect::<Vec<_>>());

    let mesh_generator = MESH_GENERATOR
        .get_or_init(|| MeshGenerator::new());

    let padded_heightmap =  HEIGHTMAP_GENERATOR
        .get_or_init(|| HeightmapGenerator::new(42))
        .generate_padded_heightmap(coord);

    let chunk_mesh = mesh_generator.generate_dual_contour_mesh(coord, &padded_heightmap, &neighborhood);

    let chunk_vertex = ChunkVertex {
        grid: coord,
        grid_x: coord.x,
        grid_z: coord.z,
        heightmap: padded_heightmap.chunk_only(),
        vertices: chunk_mesh.vertices,
        normals: chunk_mesh.normals,
    };

    let chunk_mesh = ChunkMesh {
        grid: coord,
        grid_x: coord.x,
        grid_z: coord.z,
        indices: chunk_mesh.indices,
        materials: chunk_mesh.materials,
    };

    match chunk_vertex_table.try_insert(chunk_vertex.clone()) {
        Ok(_) => {
            // successfully inserted new chunk
        }
        Err(_) => {
            // Chunk already existed—overwrite via update if desired
            // chunk_vertex_table.update(chunk_vertex.clone());
        }
    }

    match chunk_mesh_table.try_insert(chunk_mesh.clone()) {
        Ok(_) => {
            // successfully inserted new mesh
        }
        Err(_) => {
            // Chunk already existed—overwrite via update if desired
            // chunk_mesh_table.update(chunk_mesh.clone());
        }
    }

    Ok(())
}
