use bevy::prelude::*;
use bevy::render::mesh::{PrimitiveTopology, Mesh, Indices};
use bevy::render::render_asset::RenderAssetUsages;

use bevy_spacetimedb::{
    StdbConnectedEvent, StdbConnection,
    ReadInsertEvent, ReadUpdateEvent,
};

use spacetimedb_sdk::{
    Table,
    TableWithPrimaryKey,
    SubscriptionHandle as _SubscriptionHandleTrait,
};

use crate::stdb::{
    SubscriptionHandle, DbConnection,
    on_chunk_requested,
    chunk_vertex_table::ChunkVertexTableAccess,
    chunk_mesh_table::ChunkMeshTableAccess,
    Mesh as TerrainMesh,
};

use colorgrad::{CustomGradient, Gradient};

use crate::terrain::{
    types::{
        XZCoords,
        ChunkVertex, ChunkMesh, 
        MinimapUi, MinimapConfig
    },
    dirtychunks::DirtyChunks,
};

#[derive(Resource)]
pub struct TerrainGradient(pub Gradient);

const HEIGHT_RANGE: f32 = 64.0;

/// Radius in chunks for subscribing
const SUB_RADIUS: i32 = 3;
const CHUNK_SIZE: i32 = 32;

/// Holds the current heightmap subscription handle
#[derive(Resource, Default)]
pub struct TerrainSubscription {
    vertex_handle: Option<SubscriptionHandle>,
    mesh_handle: Option<SubscriptionHandle>,
    last_center: Option<XZCoords>,
}

/// System: subscribe to heightmap_chunk filtered by player position
pub fn terrain_subscription_system(
    mut c_evt: EventReader<StdbConnectedEvent>,
    cam_q: Query<&Transform, With<Camera3d>>,
    stdb: Res<StdbConnection<DbConnection>>,
    minimap_config: Res<MinimapConfig>,
    mut sub: ResMut<TerrainSubscription>,
    mut dirty_chunks: ResMut<DirtyChunks>,
) {
    // if connected or moved across chunk boundary
    let reconnect = c_evt.read().next().is_some();

    if let Ok(transform) = cam_q.single() {
        let cx = (transform.translation.x / CHUNK_SIZE as f32).floor() as i32;
        let cz = (transform.translation.z / CHUNK_SIZE as f32).floor() as i32;
        let center = XZCoords { x: cx, z: cz };

        if reconnect {
            // we don't want to do this on chunk change, only on connection
            dirty_chunks.populate_radius(center.clone());
            // dirty_chunks.mark_dirty(ChunkCoords { x: 5, z: -4 });
        }

        let changed = reconnect || sub.last_center.clone().map_or(true, |c| c != center);
        if changed {
            // as per spacetime docs, we should subscribe before unsubscribing
            // "This is because SpacetimeDB subscriptions are zero-copy. Subscribing to the same query more than once doesn't incur additional processing or serialization overhead."
            dirty_chunks.populate_radius(center.clone());

            // compute bounds
            let minimap_radius = 3; //minimap_config.radius as i32;
            let (min_x, max_x) = (cx - minimap_radius, cx + minimap_radius);
            let (min_z, max_z) = (cz - minimap_radius, cz + minimap_radius);
            let predicate = format!(
                "WHERE chunk_vertex.grid_x >= {} AND chunk_vertex.grid_x <= {} AND chunk_vertex.grid_z >= {} AND chunk_vertex.grid_z <= {}",
                min_x, max_x, min_z, max_z
            );
            // subscribe
            let vertex_handle = stdb
                .subscribe()
                .on_applied(|ctx| {
                    info!("Subscribed to terrain: {} chunks", ctx.db.chunk_vertex().count());
                })
                .on_error(|_, e| error!("Terrain sub error: {}", e))
                .subscribe(format!("SELECT * FROM chunk_vertex {}", predicate));

            let predicate = format!(
                "WHERE chunk_mesh.grid_x >= {} AND chunk_mesh.grid_x <= {} AND chunk_mesh.grid_z >= {} AND chunk_mesh.grid_z <= {}",
                min_x, max_x, min_z, max_z
            );
            let mesh_handle = stdb
                .subscribe()
                .on_applied(|ctx| {
                    info!("Subscribed to terrain meshes: {}", ctx.db.chunk_mesh().count());
                })
                .on_error(|_, e| error!("Terrain sub error: {}", e))
                .subscribe(format!("SELECT * FROM chunk_mesh {}", predicate));

            
            if let Some(h) = sub.vertex_handle.take() {
                let _ = h.unsubscribe();
            }

            if let Some(h) = sub.mesh_handle.take() {
                let _ = h.unsubscribe();
            }

            // store state
            sub.vertex_handle = Some(vertex_handle);
            sub.mesh_handle = Some(mesh_handle);
            sub.last_center = Some(center);
        }
    }
}

pub fn on_chunk_insert(
    mut events: ReadInsertEvent<ChunkVertex>,
    mut dirty_chunks: ResMut<DirtyChunks>,
) {
    for event in events.read() {
        info!("Chunk inserted: {:?}   Marking dirty", event.row.grid);
        dirty_chunks.mark_dirty(event.row.grid.clone());
    }
}

pub fn on_chunk_update(
    mut events: ReadUpdateEvent<ChunkVertex>,
    mut dirty_chunks: ResMut<DirtyChunks>,
) {
    for event in events.read() {
        info!("Chunk updated: {:?}   Marking dirty", event.new.grid);
        dirty_chunks.mark_dirty(event.new.grid.clone());
    }
}

pub fn on_mesh_insert(
    mut events: ReadInsertEvent<Mesh>,
) {
    for event in events.read() {
        info!("Mesh inserted: {:?}", event.row);
    }
}

pub fn on_mesh_update(
    mut events: ReadUpdateEvent<Mesh>,      
) {
    for event in events.read() {
        info!("Mesh updated: {:?}", event.new);
    }
}

pub fn render_terrain(
    minimap_q: Query<&MinimapUi>,
    sub: Res<TerrainSubscription>,
    minimap_config: Res<MinimapConfig>,
    gradient_res: Res<TerrainGradient>,
    stdb: Res<StdbConnection<DbConnection>>,
    mut dirty_chunks: ResMut<DirtyChunks>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut commands: Commands,
) {
    if dirty_chunks.is_empty() {
        return;
    }

    // Acquire the minimap texture handle
    let minimap_handle = minimap_q.single().unwrap().0.clone();
    let image = images.get_mut(&minimap_handle).unwrap();
    let data = image.data.as_mut().expect("Image data buffer missing");

    // Compute texture grid parameters
    let radius = minimap_config.radius as i32;
    let chunk_size = minimap_config.chunk_size;
    let tex_size = minimap_config.texture_size;

    // Determine center chunk
    let center = sub.last_center.clone().unwrap_or(XZCoords { x: 0, z: 0 });

    let mesh_table = stdb.db().chunk_mesh();
    let vertex_table = stdb.db().chunk_vertex();

    while let Some(coords) = dirty_chunks.pop_dirty() {
        if let Some(chunk_vertex) = vertex_table.iter().find(|row| row.grid == coords) {
            info!("Chunk found: {:?}", coords);
            let heightmap = chunk_vertex.heightmap.clone();
            match mesh_table.iter().find(|row| row.grid == coords) {
                Some(chunk_mesh) => {
                    render_chunk(&mut commands, &mut meshes, &mut materials, chunk_vertex, chunk_mesh, coords.clone());
                }
                None => {
                    dirty_chunks.schedule_retry(coords.clone(), 1.0);
                    info!("No mesh found for chunk: {:?}", coords);
                }
            }

            // Generate RGBA bytes functionally, then write each row in one go.
            let pixel_bytes: Vec<u8> = heightmap
                .iter()
                .flat_map(|&height| {
                    let normalized = (height / HEIGHT_RANGE).clamp(0.0, 1.0) as f64;
                    let color = gradient_res.0.at(normalized);
                    // Expand into an array, which IntoIterator flattens
                    [
                        (color.r * 255.0) as u8,
                        (color.g * 255.0) as u8,
                        (color.b * 255.0) as u8,
                        255u8,
                    ]
                })
                .collect();

            // Calculate the offset to center the chunk on the texture
            let offset_x = ((coords.x - center.x) * chunk_size as i32) + (tex_size as i32 - chunk_size as i32) / 2;
            let offset_z = ((coords.z - center.z) * chunk_size as i32) + (tex_size as i32 - chunk_size as i32) / 2;

            // info!("Offset x/z: {}, {}", offset_x, offset_z);

            // Copy each row slice into the image buffer with consistent usize indexing
            let row_stride = (chunk_size * 4) as usize;
            pixel_bytes
                .chunks(row_stride)
                .enumerate()
                .for_each(|(row_z, row_bytes)| {
                    let dest_z = (offset_z + row_z as i32) as usize;
                    let dest_x = (offset_x)  as usize;
                    if dest_z < tex_size as usize && dest_x < tex_size as usize {
                        let dest = (dest_z * tex_size as usize + dest_x) * 4;
                        // info!("Copying row {} to dest: ({},{}) {}", row_z, dest_x, dest_z, dest);
                        data[dest..dest + row_bytes.len()].copy_from_slice(row_bytes);
                    } else {
                        info!("Skipping copy of row {}: ({},{})", row_z, dest_x, dest_z);
                    }
                });
        } else {
            // No chunk yet for these coordinates, retry later
            let _ = stdb.conn().reducers.on_chunk_requested(coords.clone());
            dirty_chunks.schedule_retry(coords.clone(), 1.0);
            info!("No chunk found for coords: {:?}", coords);
        }
    }
}

fn render_chunk(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    chunk_vertex: ChunkVertex,
    chunk_mesh: ChunkMesh,
    coords: XZCoords,
) {
    info!("Mesh found: {:?}", chunk_mesh.grid);
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD);
    let positions = chunk_vertex.vertices
        .chunks_exact(3)
        .map(|c| [c[0], c[1], c[2]])
        .collect::<Vec<_>>();
    
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);

    let normals = chunk_vertex.normals
        .chunks_exact(3)
        .map(|c| [c[0], c[1], c[2]])
        .collect::<Vec<_>>();
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

    // mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; pos_len]);

    mesh.insert_indices(Indices::U32(chunk_mesh.indices));


    let mesh_handle = meshes.add(mesh);
    let material_handle = materials.add(StandardMaterial {
        base_color: Srgba::hex("#ffd891").unwrap().into(),
        metallic: 0.5,
        perceptual_roughness: 0.5,
        // double_sided: true,
        // unlit: true,
        ..default()
    });

    // Calculate the chunk's world position
    // let chunk_world_x = coords.x as f32 * CHUNK_SIZE as f32;
    // let chunk_world_z = coords.z as f32 * CHUNK_SIZE as f32;
    // let transform = Transform::from_xyz(chunk_world_x, 0.0, chunk_world_z);

    // info!("Spawning mesh {:?} at {:?}", mesh_handle, transform.translation);

    // --- Explicit AABB for Culling Debug ---
    // let chunk_size_half = CHUNK_SIZE as f32 / 2.0;
    // // Center the AABB on the chunk's local origin, Extents cover the chunk size + generous height
    // let aabb = Aabb::from_min_max(
    //     Vec3::new(0.0, 0.0, 0.0), // Min corner (local space)
    //     Vec3::new(CHUNK_SIZE as f32, HEIGHT_RANGE * 2.0, CHUNK_SIZE as f32) // Max corner (local space)
    // );
    // TODO: continue storing AABB in the mesh table
    // --- End AABB Debug ---

    commands.spawn((
        Mesh3d(mesh_handle.clone()),
        MeshMaterial3d(material_handle.clone()),
        // transform, // Use calculated transform
        Transform::IDENTITY,
        Name::new(format!("TerrainChunk_{}_{}", coords.x, coords.z)),
    ));
}

pub fn setup_minimap_gradient(mut commands: Commands) {
    let gradient = CustomGradient::new()
        .colors(&[
            colorgrad::Color::new(0.0, 0.0, 0.5, 1.0),
            colorgrad::Color::new(0.0, 0.0, 1.0, 1.0),
            colorgrad::Color::new(0.9, 0.9, 0.2, 1.0),
            colorgrad::Color::new(0.0, 0.6, 0.0, 1.0),
            colorgrad::Color::new(0.5, 0.3, 0.0, 1.0),
            colorgrad::Color::new(1.0, 1.0, 1.0, 1.0),
        ])
        .domain(&[0.0, 0.3, 0.35, 0.4, 0.8, 1.0])
        .build()
        .unwrap();
    commands.insert_resource(TerrainGradient(gradient));
}