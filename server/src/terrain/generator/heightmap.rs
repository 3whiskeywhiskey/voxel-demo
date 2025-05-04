use noise::{NoiseFn, Perlin};

use crate::terrain::coords::{ChunkCoords, CHUNK_SIZE};

const HEIGHT_RANGE: f32 = 32.0;

#[derive(Clone)]
pub struct PaddedHeightmap {
    data: Vec<f32>,
    dim: usize, // CHUNK_SIZE + 2
}

impl PaddedHeightmap {
    pub fn new(data: Vec<f32>, chunk_size: usize) -> Self {
        assert!(data.len() == (chunk_size + 2) * (chunk_size + 2));
        Self { data, dim: chunk_size + 2 }
    }

    pub fn get(&self, x: usize, z: usize) -> f32 {
        let u = (x + 1) as usize;
        let v = (z + 1) as usize;
        self.data[u + v * self.dim]
    }
}

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
        
        for z in 0..CHUNK_SIZE as i32 {
            for x in 0..CHUNK_SIZE as i32 {
                let world_pos = coord.to_world_pos(x, z);
                let height = self.sample_height(world_pos.x as f64, world_pos.z as f64);
                heights.push(height);
            }
        }
        
        heights
    }


    // generates a heightmap that is a chunk but one block larger on all sides
    // this helps the mesh generator blend between chunks
    pub fn generate_padded_heightmap(&self, coord: ChunkCoords) -> PaddedHeightmap {
        let mut heights = Vec::with_capacity(CHUNK_SIZE * CHUNK_SIZE);

        for z in -1..CHUNK_SIZE as i32 +1 {
            for x in -1..CHUNK_SIZE as i32 +1 {
                let world_pos = coord.to_world_pos(x, z);
                let height = self.sample_height(world_pos.x as f64, world_pos.z as f64);
                // if x < 5 && z < 5 {
                //     debug!("Generated height at world ({}, {}): {}", world_pos.x, world_pos.z, height);
                // }
                heights.push(height);
            }
        }

        PaddedHeightmap::new(heights, CHUNK_SIZE)
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
        let height = normalized * HEIGHT_RANGE;
        
        // if x < 5.0 && z < 5.0 {
        //     debug!("Sampled height at ({}, {}): {} (normalized: {})", x, z, height, normalized);
        // }
        
        height
    }
}