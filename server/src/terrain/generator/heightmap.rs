use noise::{NoiseFn, Perlin};

use crate::terrain::coords::{ChunkCoords, CHUNK_SIZE};

const HEIGHT_RANGE: f32 = 64.0;

pub struct HeightmapGenerator {
    noise: Perlin,
    base_frequency: f64,
    octaves: usize,
    persistence: f64,
    lacunarity: f64,
}

impl HeightmapGenerator {
    pub fn new(seed: u32) -> Self {
        Self {
            noise: Perlin::new(seed),
            base_frequency: 0.01,
            octaves: 4,
            persistence: 0.5,
            lacunarity: 2.0,
        }
    }

    pub fn generate_chunk(&self, coord: ChunkCoords) -> Vec<f32> {
        let mut heights = Vec::with_capacity(CHUNK_SIZE * CHUNK_SIZE);
        
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let world_pos = coord.to_world_pos(x, z);
                let height = self.sample_height(world_pos.x as f64, world_pos.z as f64);
                heights.push(height);
            }
        }
        
        heights
    }

    fn sample_height(&self, x: f64, z: f64) -> f32 {
        let mut amplitude = 1.0;
        let mut frequency = self.base_frequency;
        let mut noise_height = 0.0;
        let mut max_value = 0.0;

        for _ in 0..self.octaves {
            let sample_x = x * frequency;
            let sample_z = z * frequency;
            
            let perlin_value = self.noise.get([sample_x, sample_z]) as f64;
            noise_height += perlin_value * amplitude;
            
            max_value += amplitude;
            amplitude *= self.persistence;
            frequency *= self.lacunarity;
        }

        let normalized = (noise_height / max_value) as f32;
        (normalized * 0.5 + 0.5) * HEIGHT_RANGE
    }
}