use clap::Parser;
use noise::{NoiseFn, Perlin};
use image::{ImageBuffer, Rgb};
use image::RgbImage;
use imageproc::drawing::draw_line_segment_mut;
use nalgebra::Vector2;
use colorgrad::Gradient;
use futures::executor::block_on;
use spacetimedb_sdk::credentials;
use spacetimedb_sdk::Identity;
use std::collections::HashMap;
use std::f32::consts::PI;

mod stdb;

use stdb::{DbConnection, MeshChunk, ChunkCoords, HeightmapChunk, Vec3};
use stdb::{on_heightmap_generated, on_mesh_generated};

type MaterialId = u8;

async fn push_chunk(
    conn: &DbConnection,
    coords: ChunkCoords,
    heights: Vec<f32>,
) -> Result<(), spacetimedb_sdk::Error> {
    // call the reducer exposed by your module:
    conn.reducers.on_heightmap_generated(coords, heights)
}

async fn push_mesh(
    conn: &DbConnection,
    chunk: MeshChunk,
) -> Result<(), spacetimedb_sdk::Error> {
    conn.reducers.on_mesh_generated(chunk)
}


const CHUNK_SIZE: usize = 32;
const HEIGHT_RANGE: f32 = 64.0;

impl ChunkCoords {
    // fn new(x: i32, z: i32) -> Self {
    //     Self { x, z }
    // }

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

struct MeshGenerator {
}

impl MeshGenerator {
    fn new() -> Self {
        Self {}
    }

    fn heightmap_to_blocky_mesh(&self, coord: ChunkCoords, heights: &[f32]) -> MeshChunk {
        let mut verts: Vec<Vec3> = Vec::new();
        let mut norms: Vec<Vec3> = Vec::new();
        let mut idxs: Vec<u32> = Vec::new();
        let mut mats: Vec<MaterialId> = Vec::new();

        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let h = heights[(z * CHUNK_SIZE + x) as usize];
                if h <= 0.0 { continue; }
                // Four corners of top quad
                let base = verts.len() as u32;
                verts.push(Vec3 { x: x as f32,     y: h, z: z as f32 });
                verts.push(Vec3 { x: (x+1) as f32, y: h, z: z as f32 });
                verts.push(Vec3 { x: x as f32,     y: h, z: (z+1) as f32 });
                verts.push(Vec3 { x: (x+1) as f32, y: h, z: (z+1) as f32 });
                // Push four copies of the normal without requiring Vec3: Copy
                norms.extend(std::iter::repeat(Vec3 { x: 0.0, y: 1.0, z: 0.0 }).take(4));
                // One material per-vertex
                mats.extend([0; 4]); // e.g. grass=0
                // Two triangles
                idxs.extend([base, base+2, base+1, base+2, base+3, base+1]);
            }
        }
        MeshChunk {
            coord,
            vertices: verts,
            normals: norms,
            indices: idxs,
            materials: mats,
        }
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

    fn wireframe_on_heightmap(
        base: &RgbImage,
        verts: &[Vec3],
        indices: &[u32],
    ) -> RgbImage {
        let mut img = base.clone();
        let (w, h) = img.dimensions();
        let color = Rgb([255, 0, 0]);
        let p = |i: usize| {
            let v = verts[i].clone();
            (
                v.x.clamp(0.0, w as f32 - 1.0),
                v.z.clamp(0.0, h as f32 - 1.0),
            )
        };
        // Build edge usage counts to identify border edges
        let mut edge_counts: HashMap<(usize, usize), usize> = HashMap::new();
        for tri in indices.chunks(3) {
            let vids = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
            for e in &[(vids[0], vids[1]), (vids[1], vids[2]), (vids[2], vids[0])] {
                let key = if e.0 < e.1 { *e } else { (e.1, e.0) };
                *edge_counts.entry(key).or_insert(0) += 1;
            }
        }
        // Draw only edges used once (border of mesh)
        for (&(a, b), &count) in edge_counts.iter() {
            if count == 1 {
                let p0 = p(a);
                let p1 = p(b);
                draw_line_segment_mut(&mut img, p0, p1, color);
            }
        }
        img
    }
    /// Render heightmap with mesh wireframe overlay
    fn save_terrain_wireframe(
        &self,
        mesh_chunks: &[MeshChunk],
        radius: i32,
        scale: u32,
        output_path: &str,
    ) {
        let chunk_count = (radius * 2 + 1) as usize;
        let total_width = chunk_count * CHUNK_SIZE;
        let total_height = chunk_count * CHUNK_SIZE;
        // Compute isometric projection dimensions
        // Cosine/sine of 30° for isometric
        let cos30 = (30.0_f32.to_radians()).cos();
        let sin30 = (30.0_f32.to_radians()).sin();
        // Width: span of chunks in X and Z directions
        let img_w = ((total_width as f32 + total_height as f32) * cos30 * scale as f32) as u32;
        // Height: vertical + base projection
        let max_h = HEIGHT_RANGE;
        let img_h = ((max_h + (total_width + total_height) as f32 * sin30) * scale as f32) as u32;
        let base_img = RgbImage::new(img_w, img_h);
        let mut img = base_img;
        for (chunk_idx, mesh_chunk) in mesh_chunks.iter().enumerate() {
            let cz = chunk_idx / chunk_count;
            let cx = chunk_idx % chunk_count;
            let offset_x = (cx * CHUNK_SIZE) as f32;
            let offset_z = (cz * CHUNK_SIZE) as f32;
            // projection for isometric wireframe
            let p = |i: usize| {
                let v = &mesh_chunk.vertices[i];
                // world coords
                let wx = v.x + offset_x;
                let wy = v.y;
                let wz = v.z + offset_z;
                // isometric projection
                let px = (wx - wz) * cos30;
                let py = wy + (wx + wz) * sin30;
                (
                    (px * scale as f32 + img_w as f32 * 0.5).clamp(0.0, img_w as f32 - 1.0),
                    ((img_h as f32 - scale as f32) - py * scale as f32).clamp(0.0, img_h as f32 - 1.0),
                )
            };
            // count edges
            let mut edge_counts: HashMap<(usize, usize), usize> = HashMap::new();
            for tri in mesh_chunk.indices.chunks(3) {
                let vids = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
                for e in &[(vids[0], vids[1]), (vids[1], vids[2]), (vids[2], vids[0])] {
                    let key = if e.0 < e.1 { *e } else { (e.1, e.0) };
                    *edge_counts.entry(key).or_insert(0) += 1;
                }
            }
            // draw border edges in isometric
            for (&(a, b), &count) in edge_counts.iter() {
                if count == 1 {
                    let p0 = p(a);
                    let p1 = p(b);
                    draw_line_segment_mut(&mut img, p0, p1, Rgb([255, 0, 0]));
                }
            }
        }
        img.save(output_path).expect("Failed to save wireframe image");
    }

    fn save_terrain_image(
        &self,
        heights: &[Vec<f32>],
        radius: i32,
        scale: u32,
        output_path: &str,
    ) {
        let chunk_count = (radius * 2 + 1) as usize;
        let total_width = chunk_count * CHUNK_SIZE;
        let total_height = chunk_count * CHUNK_SIZE;
        let img_w = total_width as u32 * scale;
        let img_h = total_height as u32 * scale;
        
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
        
        // Create base image at scaled resolution
        let mut img = RgbImage::new(img_w, img_h);
        for z in 0..total_height {
            for x in 0..total_width {
                let color = {
                    let h = combined_heights[z * total_width + x];
                    let c = self.gradient.at((h / HEIGHT_RANGE) as f64);
                    Rgb([(c.r * 255.0) as u8, (c.g * 255.0) as u8, (c.b * 255.0) as u8])
                };
                for dy in 0..scale {
                    for dx in 0..scale {
                        img.put_pixel(x as u32 * scale + dx, z as u32 * scale + dy, color);
                    }
                }
            }
        }
        img.save(output_path)
            .expect("Failed to save image");
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

    /// Enable wireframe overlay on heightmap
    #[arg(long)]
    wireframe: bool,

    /// Pixels per world unit when rendering the image
    #[arg(long, default_value_t = 10)]
    scale: u32,
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

    let generator = HeightmapGenerator::new(args.seed);
    let mesh_generator = MeshGenerator::new();
    let visualizer = TerrainVisualizer::new();

    // Load saved token if available
    let mut builder = DbConnection::builder()
        .with_uri("https://spacetime.whiskey.works")
        .with_module_name("realm1");
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
    let mut all_meshes: Vec<MeshChunk> = Vec::new();
    
    for z in -args.radius..=args.radius {
        for x in -args.radius..=args.radius {
            let coord = ChunkCoords { x, z };
            let heights = generator.generate_chunk(coord.clone());
            log::info!("Generated chunk at ({}, {})", x, z);

            let height_chunk = HeightmapChunk {
                coord: coord.clone(),
                chunk_x: coord.x,
                chunk_z: coord.z,
                heights: heights.clone(),
            };

            // TODO: how do we parallelize these? silly to block on each sequentially innit?

            // Push to SpacetimeDB
            block_on(push_chunk(&conn, height_chunk.coord, height_chunk.heights)).expect("Failed to push chunk");

            let mesh_chunk = mesh_generator.heightmap_to_blocky_mesh(coord.clone(), &heights);
            block_on(push_mesh(&conn, mesh_chunk.clone())).expect("Failed to push chunk");
            all_meshes.push(mesh_chunk);

            // Store for local visualization
            all_heights.push(heights);
        }
    }

    log::info!("Saving terrain visualization to {}...", args.output);
    if args.wireframe {
        visualizer.save_terrain_wireframe(&all_meshes, args.radius, args.scale, &args.output);
    } else {
        visualizer.save_terrain_image(&all_heights, args.radius, args.scale, &args.output);
    }
    log::info!("Done!");
}