use std::collections::HashSet;
use bevy::prelude::*;
use std::time::Duration;
use bevy::prelude::Timer;
use bevy::prelude::TimerMode;
use crate::terrain::types::ChunkCoords;

#[derive(Resource, Default)]
pub struct DirtyChunks {
    set: HashSet<ChunkCoords>,
    radius: u8,
    retries: Vec<(ChunkCoords, Timer)>,
}

impl DirtyChunks {
    pub fn new(radius: u8) -> Self {
        Self {
            set: HashSet::new(),
            radius: radius,
            retries: Vec::new(),
        }
    }

    /// Mark a single chunk dirty
    pub fn mark_dirty(&mut self, coord: ChunkCoords) {
        info!("Marking chunk {:?} as dirty", coord);
        self.set.insert(coord);
    }

    /// Populate the set with *all* coords in the square around `center`
    pub fn populate_radius(&mut self, center: ChunkCoords) {
        self.set.clear();
        let r = self.radius;
        for x in (center.x - r as i32)..=(center.x + r as i32) {
            for z in (center.z - r as i32)..=(center.z + r as i32) {
                self.mark_dirty(ChunkCoords { x, z });
            }
        }
    }

    /// Pop *one* dirty coord (you can also drain the whole set if you prefer)
    pub fn pop_dirty(&mut self) -> Option<ChunkCoords> {
        self.set.iter().cloned().next().map(|coord| {
            self.set.remove(&coord);
            coord
        })
    }

    // is dirty
    pub fn is_dirty(&self, coord: ChunkCoords) -> bool {
        self.set.contains(&coord)
    }

    // pop if dirty
    pub fn pop_if_dirty(&mut self, coord: ChunkCoords) -> Option<ChunkCoords> {
        if self.set.contains(&coord) {
            self.set.remove(&coord);
            Some(coord)
        } else {
            None
        }
    }

    /// Quick check for our hot loop
    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }

    /// Schedule a retry for a coord after a delay
    pub fn schedule_retry(&mut self, coord: ChunkCoords, delay_secs: f32) {
        self.retries.push((
            coord,
            Timer::from_seconds(delay_secs, TimerMode::Once),
        ));
    }

    /// Tick retry timers, re-marking finished coords dirty
    pub fn tick_retries(&mut self, delta: Duration) {
        let mut to_mark = Vec::new();
        self.retries.retain_mut(|(coord, timer)| {
            timer.tick(delta);
            if timer.finished() {
                to_mark.push(coord.clone());
                false
            } else {
                true
            }
        });
        for coord in to_mark {
            self.mark_dirty(coord);
        }
    }
}

pub fn dirtychunks_tick_system(
    time: Res<Time>,
    mut dirty_chunks: ResMut<DirtyChunks>,
) {
    dirty_chunks.tick_retries(time.delta());
}