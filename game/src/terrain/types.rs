use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct MinimapConfig {
    pub radius: i32,
    pub chunk_size: f32,
    pub pixel_size: f32,
}
// Default impl sets up typical radius, sizes, etc.

#[derive(Resource, Default)]
pub struct MinimapImage(pub Handle<Image>);

// reuse your HeightmapChunk and ChunkCoords from the generated stdb module
pub use crate::stdb::{HeightmapChunk, ChunkCoords};

#[derive(Component)]
pub struct MinimapUi(pub Handle<Image>);