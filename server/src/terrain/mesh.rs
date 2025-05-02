// src/mesh.rs

use spacetimedb::{table, reducer, ReducerContext, Table};


use crate::terrain::coords::{ChunkCoords, MaterialId, Vec3};

#[table(name = mesh_chunk, public)]
pub struct MeshChunk {
    #[primary_key]
    pub coord: ChunkCoords,
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub indices: Vec<u32>,
    pub materials: Vec<MaterialId>,
}


#[reducer]
pub fn on_mesh_generated(ctx: &ReducerContext, chunk: MeshChunk) -> Result<(), String> {
    ctx.db.mesh_chunk().insert(chunk);
    Ok(())
}