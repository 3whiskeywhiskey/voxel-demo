// src/material.rs

use spacetimedb::{table, reducer, ReducerContext, Table};
use crate::terrain::coords::MaterialId;

#[table(name = material_definition)]
#[derive(Clone, Debug)]
pub struct MaterialDefinition {
    #[primary_key]
    pub id: MaterialId,
    pub name: String,
    pub base_color: Vec<f32>,
    pub texture: Option<String>,
}

#[reducer]
pub fn on_material_defined(ctx: &ReducerContext, e: MaterialDefinition) -> Result<(), String> {
    ctx.db.material_definition().insert(e);
    Ok(())
}