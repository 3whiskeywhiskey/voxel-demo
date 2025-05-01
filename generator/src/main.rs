use clap::Parser;
use noise::{NoiseFn, Perlin};
use image::{ImageBuffer, Rgb};
use nalgebra::Vector2;
use colorgrad::Gradient;
use futures::executor::block_on;
use spacetimedb_sdk::credentials;
use spacetimedb_sdk::Identity;

mod module_bindings;

use module_bindings::DbConnection;
use module_bindings::on_heightmap_generated;
use module_bindings::ChunkCoords;

async fn push_chunk(
    conn: &DbConnection,
    coord: ChunkCoords,
    heights: Vec<f32>,   // length == 32*32
) -> Result<(), spacetimedb_sdk::Error> {
    // call the reducer exposed by your module:
    conn.reducers.on_heightmap_generated(coord, heights)
}


const CHUNK_SIZE: usize = 32;
const HEIGHT_RANGE: f32 = 64.0;

impl ChunkCoords {
    fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    fn to_world_pos(&self, local_x: usize, local_z: usize) -> Vector2<f32> {
        Vector2::new(
            (self.x * CHUNK_SIZE as i32 + local_x as i32) as f32,
            (self.z * CHUNK_SIZE as i32 + local_z as i32) as f32,
        )
    }
}

struct HeightmapGenerator {
    noise: Perlin,
    base_frequency: f64,
    octaves: usize,
    persistence: f64,
    lacunarity: f64,
}

impl HeightmapGenerator {
    fn new(seed: u32) -> Self {
        Self {
            noise: Perlin::new(seed),
            base_frequency: 0.01,
            octaves: 4,
            persistence: 0.5,
            lacunarity: 2.0,
        }
    }

    fn generate_chunk(&self, coord: ChunkCoords) -> Vec<f32> {
        let mut heights = Vec::with_capacity(CHUNK_SIZE * CHUNK_SIZE);
        
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let world_pos = coord.to_world_pos(x, z);
                let height = self.sample_height(world_pos.x as f64, world_pos.y as f64);
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

struct TerrainVisualizer {
    gradient: Gradient,
}

impl TerrainVisualizer {
    fn new() -> Self {
        let gradient = colorgrad::CustomGradient::new()
            .colors(&[
                colorgrad::Color::new(0.0, 0.0, 0.5, 1.0),   // Deep water
                colorgrad::Color::new(0.0, 0.0, 1.0, 1.0),   // Shallow water
                colorgrad::Color::new(0.9, 0.9, 0.2, 1.0),   // Beach
                colorgrad::Color::new(0.0, 0.6, 0.0, 1.0),   // Grass
                colorgrad::Color::new(0.5, 0.3, 0.0, 1.0),   // Mountain
                colorgrad::Color::new(1.0, 1.0, 1.0, 1.0),   // Snow
            ])
            .domain(&[0.0, 0.3, 0.35, 0.4, 0.8, 1.0])
            .build()
            .unwrap();

        Self { gradient }
    }

    fn create_image(&self, heights: &[f32], width: usize, height: usize) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        let max_height = HEIGHT_RANGE;
        ImageBuffer::from_fn(width as u32, height as u32, |x, y| {
            let height = heights[y as usize * width + x as usize];
            let normalized = height / max_height;
            let color = self.gradient.at(normalized as f64);
            Rgb([
                (color.r * 255.0) as u8,
                (color.g * 255.0) as u8,
                (color.b * 255.0) as u8,
            ])
        })
    }

    fn save_terrain_image(&self, heights: &[Vec<f32>], radius: i32, output_path: &str) {
        let chunk_count = (radius * 2 + 1) as usize;
        let total_width = chunk_count * CHUNK_SIZE;
        let total_height = chunk_count * CHUNK_SIZE;
        
        let mut combined_heights = vec![0.0; total_width * total_height];
        
        for (chunk_idx, chunk_heights) in heights.iter().enumerate() {
            let chunk_z = chunk_idx / chunk_count;
            let chunk_x = chunk_idx % chunk_count;
            
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let src_idx = z * CHUNK_SIZE + x;
                    let dst_x = chunk_x * CHUNK_SIZE + x;
                    let dst_z = chunk_z * CHUNK_SIZE + z;
                    let dst_idx = dst_z * total_width + dst_x;
                    combined_heights[dst_idx] = chunk_heights[src_idx];
                }
            }
        }
        
        let image = self.create_image(&combined_heights, total_width, total_height);
        image.save(output_path).expect("Failed to save image");
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The SpacetimeDB module name
    #[arg(short, long, default_value = "voxel-demo")]
    module: String,

    /// The seed for terrain generation
    #[arg(short, long, default_value = "42")]
    seed: u32,

    /// The radius of chunks to generate around the origin
    #[arg(short, long, default_value = "1")]
    radius: i32,

    /// The output PNG file path
    #[arg(short, long, default_value = "target/terrain.png")]
    output: String,
}

/// Returns a file-backed credential store for this module
fn creds_store() -> credentials::File {
    credentials::File::new("voxel-demo-backend")
}

/// on_connect callback: save fresh token to disk
fn on_connected(_ctx: &DbConnection, _identity: Identity, token: &str) {
    if let Err(e) = creds_store().save(token) {
        eprintln!("Failed to save credentials: {:?}", e);
    }
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    // let client = Client::new(&args.module).unwrap();
    let generator = HeightmapGenerator::new(args.seed);
    let visualizer = TerrainVisualizer::new();

    // Load saved token if available
    let mut builder = DbConnection::builder()
        .with_uri("https://spacetime.whiskey.works")
        .with_module_name("voxel-demo-backend");
    if let Some(token) = creds_store().load().ok() {
        builder = builder.with_token(token);
    }
    let conn = builder
        .on_connect(on_connected)
        .on_connect_error(|_err_ctx, err| {
            eprintln!("Connection failed: {}", err);
        })
        .on_disconnect(|_err_ctx, maybe_err| {
            eprintln!("Disconnected: {:?}", maybe_err);
        })
        .build()
        .expect("failed to connect");
    conn.run_threaded();

    log::info!("Generating terrain with radius {}...", args.radius);
    
    let mut all_heights = Vec::new();
    
    for z in -args.radius..=args.radius {
        for x in -args.radius..=args.radius {
            let coord = ChunkCoords { x, z };
            let heights = generator.generate_chunk(coord.clone());
            log::info!("Generated chunk at ({}, {})", x, z);
            // Clone for database push to avoid move conflicts
            let heights_for_db = heights.clone();
            let coord_for_db = coord.clone();
            // Push to SpacetimeDB
            block_on(push_chunk(&conn, coord_for_db, heights_for_db)).expect("Failed to push chunk");
            // Store for local visualization
            all_heights.push(heights);
        }
    }

    log::info!("Saving terrain visualization to {}...", args.output);
    visualizer.save_terrain_image(&all_heights, args.radius, &args.output);
    log::info!("Done!");
}