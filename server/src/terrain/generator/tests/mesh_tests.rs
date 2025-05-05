use crate::terrain::{
    coords::{XZCoords, CHUNK_SIZE},
    generator::{MeshGenerator, PaddedHeightmap},
    ChunkVertex,
};
use nalgebra::Vector3;
use log::{info, debug};
use env_logger;

// Helper function to create a padded heightmap from a regular heightmap
fn create_padded_heightmap(heights: Vec<f32>) -> PaddedHeightmap {
    let cs = CHUNK_SIZE as usize;
    let padded_size = cs + 3;
    let mut padded_data = vec![heights[0]; padded_size * padded_size];  // Initialize with first height
    
    // Copy the input heights into the center of the padded data
    for z in 0..cs {
        for x in 0..cs {
            let src_idx = z * cs + x;
            let dst_idx = (z + 1) * padded_size + (x + 1);
            padded_data[dst_idx] = heights[src_idx];
        }
    }
    
    // Fill the padding by extending edge values
    // Left and right columns (including corners)
    for z in 0..padded_size {
        if z == 0 {
            // Top row padding
            padded_data[0] = heights[0]; // Left corner
            for x in 1..padded_size-1 {
                let src_x = (x - 1).min(cs - 1);
                padded_data[x] = heights[src_x];
            }
            padded_data[padded_size-1] = heights[cs-1]; // Right corner
        } else if z == padded_size-1 {
            // Bottom row padding
            let last_row_start = (cs - 1) * cs;
            padded_data[z * padded_size] = heights[last_row_start]; // Left corner
            for x in 1..padded_size-1 {
                let src_x = (x - 1).min(cs - 1);
                padded_data[z * padded_size + x] = heights[last_row_start + src_x];
            }
            padded_data[z * padded_size + padded_size-1] = heights[last_row_start + cs - 1]; // Right corner
        } else if z > 0 && z < padded_size-1 {
            // Middle rows padding
            let src_z = (z - 1).min(cs - 1);
            padded_data[z * padded_size] = heights[src_z * cs]; // Left edge
            padded_data[z * padded_size + padded_size-1] = heights[src_z * cs + cs - 1]; // Right edge
        }
    }
    
    // Fill the remaining padding cells with the nearest edge value
    for z in 0..padded_size {
        for x in 0..padded_size {
            if z == 0 || z == padded_size-1 || x == 0 || x == padded_size-1 {
                let src_z = if z == 0 { 0 } else if z == padded_size-1 { cs - 1 } else { (z - 1).min(cs - 1) };
                let src_x = if x == 0 { 0 } else if x == padded_size-1 { cs - 1 } else { (x - 1).min(cs - 1) };
                padded_data[z * padded_size + x] = heights[src_z * cs + src_x];
            }
        }
    }
    
    PaddedHeightmap::new(padded_data, CHUNK_SIZE)
}

#[test]
fn test_dual_contour_empty_heightmap() {
    let generator = MeshGenerator::new();
    // Use a height far below the surface to ensure no isosurface crossings
    let heights = vec![-1000.0; CHUNK_SIZE as usize * CHUNK_SIZE as usize];
    let padded = create_padded_heightmap(heights);
    let mesh = generator.generate_dual_contour_mesh(
        XZCoords { x: 0, z: 0 },
        &padded,
        &Vec::new()
    );
    
    assert_eq!(mesh.vertices.len(), 0, "Empty heightmap should produce no vertices");
    assert_eq!(mesh.indices.len(), 0, "Empty heightmap should produce no indices");
    assert_eq!(mesh.normals.len(), 0, "Empty heightmap should produce no normals");
    assert_eq!(mesh.materials.len(), 0, "Empty heightmap should produce no materials");
}

#[test]
fn test_dual_contour_single_height() {
    let generator = MeshGenerator::new();
    let mut heights = vec![-16.0; CHUNK_SIZE as usize * CHUNK_SIZE as usize];  // Below ground
    
    // Create a 2x2 block of positive values to ensure we get a proper isosurface
    heights[0] = 16.0;  // Above ground
    heights[1] = 16.0;
    heights[CHUNK_SIZE as usize] = 16.0;
    heights[CHUNK_SIZE as usize + 1] = 16.0;
    
    let padded = create_padded_heightmap(heights);
    let mesh = generator.generate_dual_contour_mesh(
        XZCoords { x: 0, z: 0 },
        &padded,
        &Vec::new()
    );
    
    // For a 2x2 block, we expect vertices and triangles
    assert!(!mesh.vertices.is_empty(), "Should generate vertices for single height");
    assert!(!mesh.indices.is_empty(), "Should generate indices for single height");
    assert!(!mesh.normals.is_empty(), "Should generate normals for single height");
    assert_eq!(mesh.vertices.len(), mesh.normals.len(), "Should have same number of vertex and normal components");
    
    // Check that normals are normalized
    for i in (0..mesh.normals.len()).step_by(3) {
        let normal = Vector3::new(mesh.normals[i], mesh.normals[i + 1], mesh.normals[i + 2]);
        assert!((normal.magnitude() - 1.0).abs() < 1e-6, "Normals should be normalized");
    }
}

#[test]
fn test_dual_contour_world_position() {
    let generator = MeshGenerator::new();
    let mut heights = vec![-16.0; CHUNK_SIZE as usize * CHUNK_SIZE as usize];  // Below ground
    
    // Create a 2x2 block of positive values
    heights[0] = 16.0;  // Above ground
    heights[1] = 16.0;
    heights[CHUNK_SIZE as usize] = 16.0;
    heights[CHUNK_SIZE as usize + 1] = 16.0;
    
    let padded = create_padded_heightmap(heights.clone());
    let coord1 = XZCoords { x: 0, z: 0 };
    let coord2 = XZCoords { x: 1, z: 0 };
    
    let mesh1 = generator.generate_dual_contour_mesh(coord1, &padded, &Vec::new());
    let mesh2 = generator.generate_dual_contour_mesh(coord2, &padded, &Vec::new());
    
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
    let mut heights = vec![0.0; CHUNK_SIZE as usize * CHUNK_SIZE as usize];
    
    // Create a height gradient that crosses zero multiple times
    for z in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            // Create height variations centered around 0
            let x_wave = 16.0 * ((x as f32) / 4.0).sin();
            let z_wave = 16.0 * ((z as f32) / 4.0).cos();
            heights[(z * CHUNK_SIZE + x) as usize] = x_wave + z_wave;
            
            if x < 5 && z < 5 {
                debug!("Height at ({}, {}): {}", x, z, heights[(z * CHUNK_SIZE + x) as usize]);
            }
        }
    }
    
    let padded = create_padded_heightmap(heights);
    
    // Print some padded heightmap values in a grid format
    debug!("Padded heightmap values (5x5 corner):");
    for z in 0..5 {
        let mut row = String::new();
        for x in 0..5 {
            row.push_str(&format!("{:6.1} ", padded.get(x as isize, z as isize)));
        }
        debug!("{}", row);
    }
    
    let mesh = generator.generate_dual_contour_mesh(XZCoords { x: 0, z: 0 }, &padded, &Vec::new());
    
    info!("Generated mesh with {} vertices and {} indices", mesh.vertices.len() / 3, mesh.indices.len());
    
    // Verify we have vertices and they form a continuous surface
    assert!(mesh.vertices.len() > 0, "Should generate vertices for height gradient");
    assert!(mesh.indices.len() > 0, "Should generate indices for height gradient");
    assert_eq!(mesh.vertices.len(), mesh.normals.len(), "Should have same number of vertex and normal components");
    
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

#[test]
fn test_dual_contour_with_neighbors() {
    let generator = MeshGenerator::new();
    let mut heights = vec![-16.0; CHUNK_SIZE as usize * CHUNK_SIZE as usize];
    
    // Create a slope that crosses chunk boundaries
    for z in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            heights[(z * CHUNK_SIZE + x) as usize] = (x as f32) - 16.0;
        }
    }
    
    let padded = create_padded_heightmap(heights.clone());
    let coord = XZCoords { x: 0, z: 0 };
    
    // Create a neighbor chunk with pre-computed vertices
    let mut neighbor_vertices = Vec::new();
    let mut neighbor_normals = Vec::new();
    let grid = CHUNK_SIZE as usize + 1;
    
    // Generate a simple grid of vertices for the neighbor
    for z in 0..grid {
        for x in 0..grid {
            let world_x = CHUNK_SIZE as f32 + x as f32;
            let world_z = z as f32;
            let height = (x as f32) - 16.0;
            
            neighbor_vertices.extend_from_slice(&[world_x, height, world_z]);
            neighbor_normals.extend_from_slice(&[0.0, 1.0, 0.0]); // Simple upward normals
        }
    }
    
    let neighbor = ChunkVertex {
        grid: XZCoords { x: 1, z: 0 },
        grid_x: 1,
        grid_z: 0,
        heightmap: heights.clone(),
        vertices: neighbor_vertices,
        normals: neighbor_normals,
    };
    
    let mesh = generator.generate_dual_contour_mesh(coord, &padded, &vec![neighbor]);
    
    assert!(!mesh.vertices.is_empty(), "Should generate vertices with neighbor");
    assert!(!mesh.indices.is_empty(), "Should generate indices with neighbor");
    assert_eq!(mesh.vertices.len(), mesh.normals.len(), "Should have same number of vertex and normal components");
    
    // Verify that vertices near the boundary align with the neighbor
    if !mesh.vertices.is_empty() {
        let mut boundary_found = false;
        for i in (0..mesh.vertices.len()).step_by(3) {
            let x = mesh.vertices[i];
            if (x - CHUNK_SIZE as f32).abs() < 0.1 {
                boundary_found = true;
                let y = mesh.vertices[i + 1];
                let _z = mesh.vertices[i + 2];
                
                // The height should be close to our slope function
                let expected_height = 0.0; // At x = CHUNK_SIZE, our slope crosses zero
                assert!((y - expected_height).abs() < 1.0, 
                    "Boundary vertex height should match slope, got {} expected {}", y, expected_height);
            }
        }
        assert!(boundary_found, "Should find vertices along chunk boundary");
    }
} 