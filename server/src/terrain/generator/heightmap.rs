use noise::{NoiseFn, Perlin};

use crate::terrain::coords::{XZCoords, CHUNK_SIZE};

const HEIGHT_RANGE: f32 = 32.0;

#[derive(Clone)]
pub struct PaddedHeightmap {
    data: Vec<f32>,
    dim: usize, // CHUNK_SIZE + 2
}

impl PaddedHeightmap {
    pub fn new(data: Vec<f32>, chunk_size: i32) -> Self {
        assert!(data.len() == ((chunk_size + 3) * (chunk_size + 3)) as usize);
        Self { data, dim: (chunk_size + 3) as usize }
    }

    pub fn get(&self, x: isize, z: isize) -> f32 {
        // map logical coord x,z to padded indices safely
        let u = (x.saturating_add(1).min(self.dim as isize - 1)) as usize;
        let v = (z.saturating_add(1).min(self.dim as isize - 1)) as usize;
        self.data[v * self.dim + u]
    }

    pub fn chunk_only(&self) -> Vec<f32> {
        // Extract the CHUNK_SIZE x CHUNK_SIZE interior from the padded (dim x dim) data
        let cs = self.dim - 2; // CHUNK_SIZE
        let mut out = Vec::with_capacity(cs * cs);
        // Iterate over each interior row (skip first, take cs rows)
        for row in self.data.chunks(self.dim).skip(1).take(cs) {
            // each row is length 'dim'; we want columns 1..dim-1
            out.extend_from_slice(&row[1..(self.dim - 1)]);
        }
        out
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

    pub fn generate_chunk(&self, coord: XZCoords) -> Vec<f32> {
        let mut heights = Vec::with_capacity(CHUNK_SIZE as usize * CHUNK_SIZE as usize);
        
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
    // this generates the heights of the CORNERS, so it's 33x33 chunk, so 35x35 padded
    // this helps the mesh generator blend between chunks
    pub fn generate_padded_heightmap(&self, coord: XZCoords) -> PaddedHeightmap {
        let mut heights = Vec::with_capacity(CHUNK_SIZE as usize * CHUNK_SIZE as usize);

        for z in -1..=CHUNK_SIZE +1 {
            for x in -1..=CHUNK_SIZE +1 {
                let world_pos = coord.to_world_pos(x, z);
                let height = self.sample_height(world_pos.x as f64, world_pos.z as f64);
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