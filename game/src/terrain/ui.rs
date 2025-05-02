use bevy::prelude::*;
use crate::terrain::types::{MinimapImage, MinimapConfig};

pub fn setup_minimap_ui(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    cfg: Res<MinimapConfig>,
) {
    // create the Image, spawn NodeBundle + ImageBundle + arrow
}