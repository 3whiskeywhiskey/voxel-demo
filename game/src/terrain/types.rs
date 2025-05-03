use bevy::prelude::*;

#[derive(Resource, Copy, Clone)]
pub struct MinimapConfig {
    pub radius: u8,
    pub chunk_size: u8,
    pub viewport_size: f32,  // Size of the display on screen
    pub texture_size: u32,   // Size of the actual minimap texture
}

impl Default for MinimapConfig {
    fn default() -> Self {
        Self {
            radius: 10,
            chunk_size: 32,
            viewport_size: 200.0,  // 200x200 viewport on screen
            texture_size: 672,   // radius 10 = 21 chunks on a side, 21 * 32 = 672
                                 
        }
    }
}

#[derive(Resource, Default)]
pub struct MinimapImage(pub Handle<Image>);

// reuse your HeightmapChunk and ChunkCoords from the generated stdb module
pub use crate::stdb::{HeightmapChunk, ChunkCoords};

#[derive(Component)]
pub struct MinimapUi(pub Handle<Image>);