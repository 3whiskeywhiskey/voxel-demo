use bevy::prelude::*;

#[allow(unused_imports)]
use bevy_spacetimedb::{
    ReadInsertEvent, ReadUpdateEvent, ReadDeleteEvent,
    ReadReducerEvent, ReducerResultEvent, StdbConnectedEvent, StdbConnection,
    StdbConnectionErrorEvent, StdbDisconnectedEvent, StdbPlugin, register_reducers, tables,
};

mod stdb;
use stdb::{
    DbConnection,
};


mod player;
use player::PlayerPlugin;

mod terrain;
use terrain::TerrainPlugin;

fn main() {
    App::new()
        .add_plugins(StdbPlugin::default()
            .with_connection(|send_connected, send_disconnected, send_connect_error, _| {
                let conn = DbConnection::builder()
                .with_module_name("realm1")
                .with_uri("https://spacetime.whiskey.works")
                .on_connect_error(move |_ctx, err| {
                    send_connect_error
                        .send(StdbConnectionErrorEvent { err })
                        .unwrap();
                })
                .on_disconnect(move |_ctx, err| {
                    send_disconnected
                        .send(StdbDisconnectedEvent { err })
                        .unwrap();
                })
                .on_connect(move |_ctx, _id, _c| {
                    send_connected.send(StdbConnectedEvent {}).unwrap();
                })
                .build()
                .expect("SpacetimeDB connection failed");

                // Do what you want with the connection here

                // This is very important, otherwise your client will never connect and receive data
                conn.run_threaded();
                conn
            })
            .with_events(|_plugin, _app, _db, _reducers| {
                // tables!(
                    // heightmap_chunk,
                    // mesh_chunk,
                // );
                // reducers.on_heightmap_generated(move |ctx, chunk| {
                //     println!("Heightmap chunk inserted: {:?}", chunk.coord);
                // });
            })
        )
        .add_plugins(DefaultPlugins)
        .add_plugins(PlayerPlugin)
        .add_plugins(TerrainPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, on_connected)
        .run();
}

fn on_connected(
    mut events: EventReader<StdbConnectedEvent>,
    _stdb: Res<StdbConnection<DbConnection>>,  // your generated DbConnection type
) {
    for _ in events.read() {
        info!("Connected to SpacetimeDB");

        // stdb.subscribe()
        //     .on_applied(|ctx| {
        //         let count = ctx.db.heightmap_chunk().count();
        //         info!("Initial heightmap chunk count: {}", count);
        //     })
        //     .on_error(|_, err| error!("Heightmap subscription error: {}", err))
        //     .subscribe("SELECT * FROM heightmap_chunk");

        // stdb.subscribe()
        //     .on_applied(|ctx| {
        //         let count = ctx.db.heightmap_chunk().count();
        //         info!("Initial mesh chunk count: {}", count);
        //     })
        //     .on_error(|_, err| error!("Mesh subscription error: {}", err))
        //     .subscribe("SELECT * FROM mesh_chunk");
    }
}


fn setup(
    mut commands: Commands,
    // meshes: ResMut<Assets<Mesh>>,
    // materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.5,
        affects_lightmapped_meshes: true,
    });

    // Directional "sun" light - spawn with components
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::FRAC_PI_4,
            -std::f32::consts::FRAC_PI_4,
            0.0,
        )),
        // GlobalTransform is required by Transform
        GlobalTransform::default(),
    ));

    // Ground plane - spawn with components
    // commands.spawn((
    //     Mesh3d(meshes.add(Plane3d::default().mesh().size(500.0, 500.0))),
    //     MeshMaterial3d(materials.add(StandardMaterial {
    //         base_color: Color::srgb(0.3, 0.5, 0.3),
    //         perceptual_roughness: 1.0,
    //         ..Default::default()
    //     })),
    //     Transform::from_xyz(0.0, 0.0, 0.0),
    //     // GlobalTransform is required by Transform
    //     GlobalTransform::default(),
    // ));

    // Camera is spawned by PlayerPlugin
}
