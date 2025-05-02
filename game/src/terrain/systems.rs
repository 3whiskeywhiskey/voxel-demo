use bevy::prelude::*;
use spacetimedb_sdk::Table;
use bevy_spacetimedb::{StdbConnectedEvent, StdbConnection};
use bevy_spacetimedb::{ReadInsertEvent, ReadUpdateEvent};
use spacetimedb_sdk::SubscriptionHandle as _SubscriptionHandleTrait;
use crate::stdb::{SubscriptionHandle, DbConnection};
use crate::stdb::heightmap_chunk_table::HeightmapChunkTableAccess;

use crate::terrain::types::{HeightmapChunk, ChunkCoords};

/// Radius in chunks for subscribing
const SUB_RADIUS: i32 = 4;
const CHUNK_SIZE: i32 = 32;

/// Holds the current heightmap subscription handle
#[derive(Resource, Default)]
pub struct TerrainSubscription {
    handle: Option<SubscriptionHandle>,
    last_center: Option<ChunkCoords>,
}

/// System: subscribe to heightmap_chunk filtered by player position
pub fn terrain_subscription_system(
    mut c_evt: EventReader<StdbConnectedEvent>,
    cam_q: Query<&Transform, With<Camera3d>>,
    stdb: Res<StdbConnection<DbConnection>>,
    mut sub: ResMut<TerrainSubscription>,
) {
    // if connected or moved across chunk boundary
    let reconnect = c_evt.read().next().is_some();
    if let Ok(transform) = cam_q.single() {
        let cx = (transform.translation.x / CHUNK_SIZE as f32).floor() as i32;
        let cz = (transform.translation.z / CHUNK_SIZE as f32).floor() as i32;
        let center = ChunkCoords { x: cx, z: cz };
        let changed = reconnect || sub.last_center.clone().map_or(true, |c| c != center);
        if changed {
            // as per spacetime docs, we should subscribe before unsubscribing
            // "This is because SpacetimeDB subscriptions are zero-copy. Subscribing to the same query more than once doesn't incur additional processing or serialization overhead."

            // compute bounds
            let (min_x, max_x) = (cx - SUB_RADIUS, cx + SUB_RADIUS);
            let (min_z, max_z) = (cz - SUB_RADIUS, cz + SUB_RADIUS);
            let sql = format!(
                "SELECT * FROM heightmap_chunk WHERE chunk_x >= {} AND chunk_x <= {} AND chunk_z >= {} AND chunk_z <= {}",
                min_x, max_x, min_z, max_z
            );
            // subscribe
            let handle = stdb
                .subscribe()
                .on_applied(|ctx| {
                    info!("Subscribed to terrain: {} chunks", ctx.db.heightmap_chunk().count());
                })
                .on_error(|_, e| error!("Terrain sub error: {}", e))
                .subscribe(sql);
            
            if let Some(h) = sub.handle.take() {
                let _ = h.unsubscribe();
            }

            // store state
            sub.handle = Some(handle);
            sub.last_center = Some(center);
        }
    }
}

pub fn on_heightmap_insert(
    mut events: ReadInsertEvent<HeightmapChunk>,
) {
    for event in events.read() {
        info!("Heightmap chunk inserted: {:?}", event.row.coord);
    }
}

pub fn on_heightmap_update(
    mut events: ReadUpdateEvent<HeightmapChunk>,
) {
    for event in events.read() {
        info!("Heightmap chunk updated: {:?}", event.new.coord);
    }
} 