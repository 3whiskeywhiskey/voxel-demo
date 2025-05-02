use bevy::prelude::*;
use bevy_spacetimedb::{InsertEvent, UpdateEvent};
use crate::terrain::systems::{terrain_subscription_system, TerrainSubscription, on_heightmap_insert, on_heightmap_update};
use crate::terrain::types::{MinimapConfig, MinimapImage};
use crate::terrain::ui::setup_minimap_ui;
use crate::terrain::types::HeightmapChunk;


pub struct TerrainPlugin;
impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app
        // Init our subscription‐handle resource
        .init_resource::<TerrainSubscription>()
        .insert_resource(MinimapConfig::default())
        .insert_resource(MinimapImage::default())

        .add_event::<InsertEvent<HeightmapChunk>>()
        .add_event::<UpdateEvent<HeightmapChunk>>()

        // UI setup
        .add_systems(Startup, setup_minimap_ui)

        // terrain event handlers
        .add_systems(
            Update,
            (
                terrain_subscription_system,
                on_heightmap_insert,
                on_heightmap_update,
                // systems::update_minimap_arrow,
            ),
        );
    }
}

