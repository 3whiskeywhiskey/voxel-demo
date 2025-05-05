use bevy::prelude::*;
use bevy_spacetimedb::{InsertEvent, UpdateEvent};
use crate::terrain::{
    types::{ChunkVertex, ChunkMesh},
    ui::setup_minimap_ui,
    types::{MinimapConfig, MinimapImage},
    dirtychunks::{DirtyChunks, dirtychunks_tick_system},
    systems::{
        TerrainSubscription, 
        terrain_subscription_system, 
        on_chunk_insert, on_chunk_update, 
        render_terrain, setup_minimap_gradient
    },
};


pub struct TerrainPlugin;
impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        let minimap_config = MinimapConfig::default();

        app
        // Init our subscription‐handle resource
        .insert_resource(minimap_config)
        .insert_resource(DirtyChunks::new(3))
        .init_resource::<TerrainSubscription>()
        .init_resource::<MinimapImage>()

        .add_event::<InsertEvent<ChunkVertex>>()
        .add_event::<UpdateEvent<ChunkVertex>>()
        .add_event::<InsertEvent<ChunkMesh>>()
        .add_event::<UpdateEvent<ChunkMesh>>()

        // UI setup
        .add_systems(Startup, (setup_minimap_ui, setup_minimap_gradient))

        // terrain event handlers
        .add_systems(
            Update,
            (
                terrain_subscription_system,
                on_chunk_insert,
                on_chunk_update,
                render_terrain,
                dirtychunks_tick_system,
                // systems::update_minimap_arrow,
            ),
        );
    }
}
