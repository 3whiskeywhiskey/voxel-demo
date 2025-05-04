use crate::terrain::{
    coords::{ChunkCoords, CHUNK_SIZE},
    generator::{MeshGenerator, PaddedHeightmap},
};
use nalgebra::Vector3;
use log::{info, debug};
use env_logger;

// Helper function to create a padded heightmap from a regular heightmap
fn create_padded_heightmap(heights: Vec<f32>) -> PaddedHeightmap {
    let mut padded_data = Vec::with_capacity((CHUNK_SIZE + 2) * (CHUNK_SIZE + 2));
    
    // Top padding row
    padded_data.extend(vec![0.0; CHUNK_SIZE + 2]);
    
    // Middle rows with side padding
    for z in 0..CHUNK_SIZE {
        padded_data.push(0.0); // Left padding
        for x in 0..CHUNK_SIZE {
            padded_data.push(heights[z * CHUNK_SIZE + x]);
        }
        padded_data.push(0.0); // Right padding
    }
    
    // Bottom padding row
    padded_data.extend(vec![0.0; CHUNK_SIZE + 2]);
    
    PaddedHeightmap::new(padded_data, CHUNK_SIZE)
}

#[test]
fn test_dual_contour_empty_heightmap() {
    let generator = MeshGenerator::new();
    let heights = vec![0.0; CHUNK_SIZE * CHUNK_SIZE];
    let padded = create_padded_heightmap(heights);
    let mesh = generator.generate_mesh(ChunkCoords { x: 0, z: 0 }, padded);
    
    assert_eq!(mesh.vertices.len(), 0, "Empty heightmap should produce no vertices");
    assert_eq!(mesh.indices.len(), 0, "Empty heightmap should produce no indices");
    assert_eq!(mesh.normals.len(), 0, "Empty heightmap should produce no normals");
    assert_eq!(mesh.materials.len(), 0, "Empty heightmap should produce no materials");
}

#[test]
fn test_dual_contour_single_height() {
    let generator = MeshGenerator::new();
    let mut heights = vec![-16.0; CHUNK_SIZE * CHUNK_SIZE];  // Below ground
    
    // Create a 2x2 block of positive values to ensure we get a proper isosurface
    heights[0] = 16.0;  // Above ground
    heights[1] = 16.0;
    heights[CHUNK_SIZE] = 16.0;
    heights[CHUNK_SIZE + 1] = 16.0;
    
    let padded = create_padded_heightmap(heights);
    let mesh = generator.generate_mesh(ChunkCoords { x: 0, z: 0 }, padded);
    
    // For a 2x2 block, we expect vertices and triangles
    assert!(!mesh.vertices.is_empty(), "Should generate vertices for single height");
    assert!(!mesh.indices.is_empty(), "Should generate indices for single height");
    assert!(!mesh.normals.is_empty(), "Should generate normals for single height");
    assert_eq!(mesh.vertices.len(), mesh.normals.len(), "Should have same number of vertex and normal components");
    assert_eq!(mesh.vertices.len() / 3, mesh.materials.len(), "Should have one material per vertex");
    
    // Check that normals are normalized
    for i in (0..mesh.normals.len()).step_by(3) {
        let normal = Vector3::new(mesh.normals[i], mesh.normals[i + 1], mesh.normals[i + 2]);
        assert!((normal.magnitude() - 1.0).abs() < 1e-6, "Normals should be normalized");
    }
}

#[test]
fn test_dual_contour_world_position() {
    let generator = MeshGenerator::new();
    let mut heights = vec![-16.0; CHUNK_SIZE * CHUNK_SIZE];  // Below ground
    
    // Create a 2x2 block of positive values
    heights[0] = 16.0;  // Above ground
    heights[1] = 16.0;
    heights[CHUNK_SIZE] = 16.0;
    heights[CHUNK_SIZE + 1] = 16.0;
    
    let padded = create_padded_heightmap(heights.clone());
    let coord1 = ChunkCoords { x: 0, z: 0 };
    let coord2 = ChunkCoords { x: 1, z: 0 };
    
    let mesh1 = generator.generate_mesh(coord1, padded.clone());
    let mesh2 = generator.generate_mesh(coord2, padded);
    
    // Find corresponding vertices in both meshes
    if !mesh1.vertices.is_empty() && !mesh2.vertices.is_empty() {
        let x1 = mesh1.vertices[0];
        let x2 = mesh2.vertices[0];
        assert!(x2 > x1, "Second chunk vertices should have larger X coordinates");
        assert!((x2 - x1 - CHUNK_SIZE as f32).abs() < CHUNK_SIZE as f32, 
                "X coordinate difference should be approximately CHUNK_SIZE");
    }
}

#[test]
fn test_dual_contour_height_gradient() {
    // Initialize logging
    let _ = env_logger::builder().is_test(true).try_init();
    
    let generator = MeshGenerator::new();
    let mut heights = vec![0.0; CHUNK_SIZE * CHUNK_SIZE];
    
    // Create a height gradient that crosses zero multiple times
    for z in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            // Create height variations centered around 0
            let x_wave = 16.0 * (x as f32 / 4.0).sin();
            let z_wave = 16.0 * (z as f32 / 4.0).cos();
            heights[z * CHUNK_SIZE + x] = x_wave + z_wave;
            
            if x < 5 && z < 5 {
                debug!("Height at ({}, {}): {}", x, z, heights[z * CHUNK_SIZE + x]);
            }
        }
    }
    
    let padded = create_padded_heightmap(heights);
    
    // Print some padded heightmap values in a grid format
    debug!("Padded heightmap values (5x5 corner):");
    for z in 0..5 {
        let mut row = String::new();
        for x in 0..5 {
            row.push_str(&format!("{:6.1} ", padded.get(x, z)));
        }
        debug!("{}", row);
    }
    
    let mesh = generator.generate_mesh(ChunkCoords { x: 0, z: 0 }, padded);
    
    info!("Generated mesh with {} vertices and {} indices", mesh.vertices.len() / 3, mesh.indices.len());
    
    // Verify we have vertices and they form a continuous surface
    assert!(mesh.vertices.len() > 0, "Should generate vertices for height gradient");
    assert!(mesh.indices.len() > 0, "Should generate indices for height gradient");
    assert_eq!(mesh.vertices.len(), mesh.normals.len(), "Should have same number of vertex and normal components");
    assert_eq!(mesh.vertices.len() / 3, mesh.materials.len(), "Should have one material per vertex");
    
    // Check that normals are normalized
    for i in (0..mesh.normals.len()).step_by(3) {
        let normal = Vector3::new(mesh.normals[i], mesh.normals[i + 1], mesh.normals[i + 2]);
        assert!((normal.magnitude() - 1.0).abs() < 1e-6, "Normals should be normalized");
    }
    
    // Print some vertex positions and normals for debugging
    for i in 0..std::cmp::min(5, mesh.vertices.len() / 3) {
        let pos = Vector3::new(
            mesh.vertices[i * 3],
            mesh.vertices[i * 3 + 1],
            mesh.vertices[i * 3 + 2]
        );
        let normal = Vector3::new(
            mesh.normals[i * 3],
            mesh.normals[i * 3 + 1],
            mesh.normals[i * 3 + 2]
        );
        debug!("Vertex {}: pos={:?}, normal={:?}", i, pos, normal);
    }
}

// Legacy tests for the blocky mesh generator
#[cfg(test)]
mod deprecated_blocky_tests {
    use super::*;

    fn create_legacy_mesh(generator: &MeshGenerator, coord: ChunkCoords, heights: Vec<f32>) -> crate::entity::Mesh {
        generator.heightmap_to_blocky_mesh(coord, heights)
    }

    #[test]
    fn test_blocky_empty_heightmap() {
        let generator = MeshGenerator::new();
        let heights = vec![0.0; CHUNK_SIZE * CHUNK_SIZE];
        let mesh = create_legacy_mesh(&generator, ChunkCoords { x: 0, z: 0 }, heights);
        
        assert_eq!(mesh.vertices.len(), 0, "Empty heightmap should produce no vertices");
        assert_eq!(mesh.indices.len(), 0, "Empty heightmap should produce no indices");
        assert_eq!(mesh.normals.len(), 0, "Empty heightmap should produce no normals");
        assert_eq!(mesh.materials.len(), 0, "Empty heightmap should produce no materials");
    }

    #[test]
    fn test_blocky_single_block() {
        let generator = MeshGenerator::new();
        let mut heights = vec![0.0; CHUNK_SIZE * CHUNK_SIZE];
        heights[0] = 1.0;
        let mesh = create_legacy_mesh(&generator, ChunkCoords { x: 0, z: 0 }, heights);
        
        // For a single block at the edge of the chunk, we expect:
        // - 4 vertices for top face
        // - 4 vertices each for front, right faces (2 faces since at chunk edge)
        // Total: 12 vertices, each with 3 coordinates
        assert_eq!(mesh.vertices.len(), 36, "Single block should have 12 vertices * 3 coordinates");
        assert_eq!(mesh.normals.len(), 36, "Single block should have 12 normals * 3 coordinates");
        assert_eq!(mesh.indices.len(), 18, "Single block should have 6 triangles * 3 indices");
        assert_eq!(mesh.materials.len(), 12, "Single block should have 12 material indices");
        
        // Verify normals are correct
        let mut normal_counts = std::collections::HashMap::new();
        for i in (0..mesh.normals.len()).step_by(3) {
            let normal = format!("{},{},{}", 
                mesh.normals[i], 
                mesh.normals[i + 1], 
                mesh.normals[i + 2]
            );
            *normal_counts.entry(normal).or_insert(0) += 1;
        }
        
        // Should have:
        // - 4 upward-facing normals (0,1,0) for top face
        // - 4 forward-facing normals (0,0,1) for front face
        // - 4 right-facing normals (1,0,0) for right face
        assert_eq!(normal_counts.get("0,1,0").unwrap_or(&0), &4, "Should have 4 upward normals");
        assert_eq!(normal_counts.get("0,0,1").unwrap_or(&0), &4, "Should have 4 forward normals");
        assert_eq!(normal_counts.get("1,0,0").unwrap_or(&0), &4, "Should have 4 right normals");
    }
} 