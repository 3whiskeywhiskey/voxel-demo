use bevy::prelude::*;
use bevy_spacetimedb::{InsertEvent, UpdateEvent};
use crate::terrain::{
    types::HeightmapChunk,
    ui::setup_minimap_ui,
    types::{MinimapConfig, MinimapImage},
    dirtychunks::{DirtyChunks, dirtychunks_tick_system},
    systems::{
        TerrainSubscription, 
        terrain_subscription_system, 
        on_heightmap_insert, on_heightmap_update, 
        render_heightmap, setup_minimap_gradient
    },
};


pub struct TerrainPlugin;
impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        let minimap_config = MinimapConfig::default();

        app
        // Init our subscription‐handle resource
        .insert_resource(minimap_config)
        .insert_resource(DirtyChunks::new(minimap_config.radius))
        .init_resource::<TerrainSubscription>()
        .init_resource::<MinimapImage>()

        .add_event::<InsertEvent<HeightmapChunk>>()
        .add_event::<UpdateEvent<HeightmapChunk>>()

        // UI setup
        .add_systems(Startup, (setup_minimap_ui, setup_minimap_gradient))

        // terrain event handlers
        .add_systems(
            Update,
            (
                terrain_subscription_system,
                on_heightmap_insert,
                on_heightmap_update,
                render_heightmap,
                dirtychunks_tick_system,
                // systems::update_minimap_arrow,
            ),
        );
    }
}
