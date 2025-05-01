// src/mesh.rs

#[cfg(feature = "server")]
use spacetimedb::{table, reducer, ReducerContext, Table};

use crate::coords::{ChunkCoords, MaterialId, Vec3};

#[cfg_attr(feature = "server", table(name = mesh_chunk))]
#[derive(Clone, Debug)]
pub struct MeshChunk {
    #[cfg_attr(feature = "server", primary_key)]
    pub coord: ChunkCoords,
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub indices: Vec<u32>,
    pub materials: Vec<MaterialId>,
}


#[cfg(feature = "server")]
#[reducer]
pub fn on_mesh_generated(ctx: &ReducerContext, chunk: MeshChunk) -> Result<(), String> {
    ctx.db.mesh_chunk().insert(chunk);
    Ok(())
}