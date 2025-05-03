use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    window::CursorGrabMode,
    core_pipeline::Skybox,
    ui::{UiRect, PositionType, JustifyContent},
};
use bevy::pbr::Atmosphere;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerSettings>()
            .add_systems(Startup, (setup_player, setup_ui))
            .add_systems(Update, (player_move, player_look, toggle_cursor, update_position_text).chain());
    }
}

#[derive(Resource)]
pub struct PlayerSettings {
    pub sensitivity: f32,
    pub speed: f32,
}

impl Default for PlayerSettings {
    fn default() -> Self {
        Self {
            sensitivity: 0.15,
            speed: 12.0,
        }
    }
}

/// Marker component for the player-controlled camera
#[derive(Component)]
pub struct PlayerController;

/// Marker component for the position text
#[derive(Component)]
pub struct PositionText;

fn setup_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load the skybox texture
    let skybox_handle: Handle<Image> = asset_server.load("environment_maps/night.ktx2"); 
    
    // Spawn 2D camera with priority 0
    commands.spawn((
        Camera2d::default(),
        Camera {
            order: 0, // This ensures 2D renders before 3D
            ..default()
        },
        Transform::default(),
        GlobalTransform::default(),
    ));
    
    // Spawn 3D camera with priority 1
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 1, // This ensures 3D renders after 2D
            hdr: true, // Atmosphere requires HDR
            clear_color: ClearColorConfig::None,
            ..default()
        },
        Transform::from_xyz(-2.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        GlobalTransform::default(),
        Atmosphere::EARTH,
        Skybox {
            image: skybox_handle.clone(),
            brightness: 500.0,
            rotation: Quat::default(),
            ..default()
        },
        PlayerController,
    ));
}

fn setup_ui(mut commands: Commands, _asset_server: Res<AssetServer>) {
    // Root node
    commands
        .spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|parent| {
            // Top bar with position
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.),
                        height: Val::Px(40.),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        padding: UiRect::all(Val::Px(10.)),
                        ..default()
                    },
                    BackgroundColor(Color::BLACK),
                ))
                .with_children(|parent| {
                    // Position text (left)
                    parent.spawn((
                        Node {
                            padding: UiRect::axes(Val::Px(5.), Val::Px(1.)),
                            ..default()
                        },
                        Text::new("Position: (0.0, 0.0, 0.0)"),
                        TextColor(Color::WHITE),
                        PositionText,
                    ));
                });

            // Crosshair in center
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.),
                    top: Val::Percent(50.),
                    width: Val::Px(10.),
                    height: Val::Px(10.),
                    margin: UiRect {
                        left: Val::Px(-5.),
                        top: Val::Px(-5.),
                        ..default()
                    },
                    ..default()
                },
                BackgroundColor(Color::WHITE),
            ));

            // Bottom panel with controls
            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(0.),
                        width: Val::Percent(100.),
                        height: Val::Px(80.),
                        padding: UiRect::all(Val::Px(10.)),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    BackgroundColor(Color::BLACK),
                ))
                .with_children(|parent| {
                    // Controls header
                    parent.spawn((
                        Node {
                            margin: UiRect::bottom(Val::Px(5.)),
                            ..default()
                        },
                        Text::new("Controls:"),
                        TextColor(Color::WHITE),
                    ));
                    
                    // Controls text
                    parent.spawn((
                        Node {
                            ..default()
                        },
                        Text::new("WASD - Move  |  Space/Shift - Up/Down  |  Esc - Toggle Mouse"),
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

fn update_position_text(
    mut query: Query<&mut Text, With<PositionText>>,
    camera_query: Query<&Transform, With<PlayerController>>,
) {
    if let (Ok(mut text), Ok(transform)) = (query.single_mut(), camera_query.single()) {
        text.0 = format!(
            "Position: ({:.1}, {:.1}, {:.1})",
            transform.translation.x,
            transform.translation.y,
            transform.translation.z
        );
    }
}

fn player_move(
    settings: Res<PlayerSettings>,
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<PlayerController>>,
) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };

    let mut direction = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::KeyW) {
        direction += *transform.forward();
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        direction += *transform.back();
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        direction += *transform.left();
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        direction += *transform.right();
    }
    if keyboard_input.pressed(KeyCode::Space) {
        direction += Vec3::Y;
    }
    if keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight) {
        direction += Vec3::NEG_Y;
    }

    if direction != Vec3::ZERO {
        // Log movement only when there's input
        // info!("Moving player: direction={:?}", direction);
        transform.translation += direction.normalize_or_zero() * settings.speed * time.delta_secs();
    } else {
        // Optional: Log when no movement keys are pressed
        // trace!("Player move system running, no input.");
    }
}


fn player_look(
    settings: Res<PlayerSettings>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut query: Query<&mut Transform, With<PlayerController>>,
    windows: Query<&Window>,
) {
    let Ok(mut transform) = query.single_mut() else { return; };
    let Ok(window) = windows.single() else { return; }; 

    // Only rotate if cursor is grabbed
    if window.cursor_options.grab_mode != CursorGrabMode::Locked {
        // Log if cursor is not locked
        // trace!("Player look: Cursor not locked.");
        mouse_motion_events.clear(); // Consume events to avoid buildup
        return;
    }

    let mut delta: Vec2 = Vec2::ZERO;
    for event in mouse_motion_events.read() {
        delta += event.delta;
    }

    if delta.length_squared() > 1e-6 { // Check for significant movement
        // Log mouse movement
        // info!("Rotating player: delta={:?}", delta);
        let sensitivity = settings.sensitivity * 0.1; // Adjust sensitivity scale
        let mut pitch = transform.rotation.to_euler(EulerRot::YXZ).1;
        let mut yaw = transform.rotation.to_euler(EulerRot::YXZ).0;

        // Apply yaw rotation (around Y-axis)
        yaw -= delta.x * sensitivity;

        // Apply pitch rotation (around X-axis), clamping to avoid looking upside down
        let max_pitch = std::f32::consts::FRAC_PI_2 - 1e-3;
        pitch = (pitch - delta.y * sensitivity).clamp(-max_pitch, max_pitch);

        transform.rotation = Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
    } else {
         // Optional: Log when no mouse motion
        // trace!("Player look system running, no mouse motion.");
    }
}


fn toggle_cursor(
    mut windows: Query<&mut Window>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        if let Ok(mut window) = windows.single_mut() { 
            match window.cursor_options.grab_mode {
                CursorGrabMode::None => {
                    window.cursor_options.grab_mode = CursorGrabMode::Locked;
                    window.cursor_options.visible = false;
                }
                _ => {
                    window.cursor_options.grab_mode = CursorGrabMode::None;
                    window.cursor_options.visible = true;
                }
            }
        }
    }
} 