use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::ui::UiRect;
use bevy::ui::widget::ImageNode;
use crate::terrain::types::{MinimapConfig, MinimapImage, MinimapUi};

pub fn setup_minimap_ui(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    cfg: Res<MinimapConfig>,
    mut minimap_image: ResMut<MinimapImage>,
) {
    // Create the minimap texture (1024x1024)
    let size = cfg.texture_size;
    let checker_size = 64; // Size of each checker square
    let mut texture_data = Vec::with_capacity((size * size * 4) as usize);
    
    // Generate checkerboard pattern
    for y in 0..size {
        for x in 0..size {
            let checker_x = (x / checker_size) % 2;
            let checker_y = (y / checker_size) % 2;
            let is_white = (checker_x + checker_y) % 2 == 0;
            
            // White squares will be white, black squares will be red
            let (r, g, b) = if is_white {
                (255, 255, 255)
            } else {
                (255, 0, 0)
            };
            
            texture_data.extend_from_slice(&[r, g, b, 255]);
        }
    }

    let mut texture = Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    
    // Set texture usage flags for UI rendering and dynamic updates
    texture.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING 
        | TextureUsages::COPY_DST 
        | TextureUsages::RENDER_ATTACHMENT;
    
    // Add texture to assets and store handle
    let texture_handle = images.add(texture);
    minimap_image.0 = texture_handle.clone();

    // Create a viewport-based container that will scale uniformly
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(20.),
                bottom: Val::Px(90.),
                width: Val::Px(cfg.viewport_size),
                height: Val::Px(cfg.viewport_size),
                border: UiRect::all(Val::Px(2.)),
                ..default()
            },
            ImageNode::new(texture_handle.clone()),
            BorderColor(Color::WHITE),
            Name::new("Minimap"),
            MinimapUi(texture_handle.clone()),
        ));
}

// Add a system to handle minimap scaling
// pub fn update_minimap_scale(
//     windows: Query<&Window>,
//     mut minimap_query: Query<&mut Node, With<MinimapUi>>,
//     cfg: Res<MinimapConfig>,
// ) {
//     if let Ok(window) = windows.single() {
//         let min_dimension = window.width().min(window.height());
//         let scale = min_dimension / cfg.texture_size as f32; // Base scale on original texture size
        
//         if let Ok(mut node) = minimap_query.single_mut() {
//             node.width = Val::Px(200.0 * scale);
//             node.height = Val::Px(200.0 * scale);
//         }
//     }
// }