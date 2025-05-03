// src/entity/mesh.rs

use spacetimedb::{
    table,
};


#[table(name = mesh, public)]
#[derive(Clone, Debug)]
pub struct Mesh {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
    pub normals: Vec<f32>,
    pub materials: Vec<u32>,
}


