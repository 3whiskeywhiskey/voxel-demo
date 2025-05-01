// src/material.rs

#[cfg(feature = "server")]
use spacetimedb::{table, reducer, ReducerContext, Table};
use crate::coords::MaterialId;

#[cfg_attr(feature = "server", table(name = material_definition))]
#[derive(Clone, Debug)]
pub struct MaterialDefinition {
    #[cfg_attr(feature = "server", primary_key)]
    pub id: MaterialId,
    pub name: String,
    pub base_color: Vec<f32>,
    pub texture: Option<String>,
}

#[cfg(feature = "server")]
#[reducer]
pub fn on_material_defined(ctx: &ReducerContext, e: MaterialDefinition) -> Result<(), String> {
    ctx.db.material_definition().insert(e);
    Ok(())
}