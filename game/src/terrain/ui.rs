use bevy::prelude::*;
// use bevy_ui::{Node, BackgroundColor, Style, PositionType, UiRect, Val, Size};
use crate::terrain::types::MinimapConfig;

pub fn setup_minimap_ui(
    mut commands: Commands,
    cfg: Res<MinimapConfig>,
) {
    // let size_px = cfg.pixel_size as f32;

    // commands.spawn((
    //     Node::default(),
    //     Style {
    //         position_type: PositionType::Absolute,
    //         position: UiRect {
    //             left: Val::Px(10.0),
    //             bottom: Val::Px(10.0),
    //             ..default()
    //         },
    //         size: Size::new(Val::Px(size_px), Val::Px(size_px)),
    //         ..default()
    //     },
    //     BackgroundColor(Color::WHITE),
    // ));
}