use crate::terrain::{
    coords::{ChunkCoords, CHUNK_SIZE},
    generator::MeshGenerator,
};

#[test]
fn test_mesh_generator_creation() {
    let generator = MeshGenerator::new();
    // Basic instantiation test - verify we can create a mesh
    let mesh = generator.generate_mesh(ChunkCoords { x: 0, z: 0 }, vec![0.0; CHUNK_SIZE * CHUNK_SIZE]);
    assert_eq!(mesh.vertices.len(), 0);
}

#[test]
fn test_empty_heightmap() {
    let generator = MeshGenerator::new();
    let heights = vec![0.0; CHUNK_SIZE * CHUNK_SIZE];
    let mesh = generator.generate_mesh(ChunkCoords { x: 0, z: 0 }, heights);
    
    assert_eq!(mesh.vertices.len(), 0, "Empty heightmap should produce no vertices");
    assert_eq!(mesh.indices.len(), 0, "Empty heightmap should produce no indices");
    assert_eq!(mesh.normals.len(), 0, "Empty heightmap should produce no normals");
    assert_eq!(mesh.materials.len(), 0, "Empty heightmap should produce no materials");
}

#[test]
fn test_single_height_block() {
    let generator = MeshGenerator::new();
    let mut heights = vec![0.0; CHUNK_SIZE * CHUNK_SIZE];
    heights[0] = 1.0; // Single block at (0,0)
    
    let mesh = generator.generate_mesh(ChunkCoords { x: 0, z: 0 }, heights);
    
    // For a single block, we expect:
    // - 4 vertices for the top face
    assert_eq!(mesh.vertices.len(), 12); // 4 vertices * 3 coordinates
    assert_eq!(mesh.normals.len(), 12); // 4 normals * 3 coordinates
    assert_eq!(mesh.indices.len(), 6); // 2 triangles * 3 indices
    assert_eq!(mesh.materials.len(), 4); // 4 vertices * 1 material id
    
    // Check that all normals are pointing up
    for i in (0..mesh.normals.len()).step_by(3) {
        assert_eq!(mesh.normals[i], 0.0); // x
        assert_eq!(mesh.normals[i + 1], 1.0); // y
        assert_eq!(mesh.normals[i + 2], 0.0); // z
    }
    
    // Check Y coordinates are at the correct height
    for i in (1..mesh.vertices.len()).step_by(3) {
        assert_eq!(mesh.vertices[i], 1.0); // All Y coordinates should be at height 1.0
    }
}

#[test]
fn test_adjacent_blocks() {
    let generator = MeshGenerator::new();
    let mut heights = vec![0.0; CHUNK_SIZE * CHUNK_SIZE];
    
    // Create two adjacent blocks
    heights[0] = 1.0;
    heights[1] = 1.0;
    
    let mesh = generator.generate_mesh(ChunkCoords { x: 0, z: 0 }, heights);
    
    // For two adjacent blocks, we expect:
    // - 8 vertices (4 per block, no sharing for top faces)
    assert_eq!(mesh.vertices.len(), 24); // 8 vertices * 3 coordinates
    assert_eq!(mesh.normals.len(), 24); // 8 normals * 3 coordinates
    assert_eq!(mesh.indices.len(), 12); // 4 triangles * 3 indices
    assert_eq!(mesh.materials.len(), 8); // 8 vertices * 1 material id
}

#[test]
fn test_world_position_offset() {
    let generator = MeshGenerator::new();
    let mut heights = vec![0.0; CHUNK_SIZE * CHUNK_SIZE];
    heights[0] = 1.0;
    
    let coord1 = ChunkCoords { x: 0, z: 0 };
    let coord2 = ChunkCoords { x: 1, z: 0 };
    
    let mesh1 = generator.generate_mesh(coord1, heights.clone());
    let mesh2 = generator.generate_mesh(coord2, heights);
    
    // Check that the x coordinates are offset by CHUNK_SIZE
    let x1 = mesh1.vertices[0];
    let x2 = mesh2.vertices[0];
    assert_eq!(x2 - x1, CHUNK_SIZE as f32);
}

#[test]
fn test_height_affects_y_position() {
    let generator = MeshGenerator::new();
    let mut heights = vec![0.0; CHUNK_SIZE * CHUNK_SIZE];
    
    // Test with different heights
    heights[0] = 1.0;
    heights[1] = 2.0;
    
    let mesh = generator.generate_mesh(ChunkCoords { x: 0, z: 0 }, heights);
    
    // Find Y coordinates of first vertices of each block
    let y1 = mesh.vertices[1]; // Y coordinate of first vertex of first block
    let y2 = mesh.vertices[13]; // Y coordinate of first vertex of second block
    
    assert_eq!(y2 - y1, 1.0, "Y coordinates should differ by height difference");
}

#[test]
fn test_winding_order() {
    let generator = MeshGenerator::new();
    let mut heights = vec![0.0; CHUNK_SIZE * CHUNK_SIZE];
    heights[0] = 1.0; // Single block at (0,0)
    
    let mesh = generator.generate_mesh(ChunkCoords { x: 0, z: 0 }, heights);
    
    // For a single quad, verify vertex positions
    assert_eq!(mesh.vertices.len(), 12); // 4 vertices * 3 coordinates
    
    // Extract vertices (each vertex is 3 floats: x,y,z)
    let v0 = &mesh.vertices[0..3];  // Top-left
    let v1 = &mesh.vertices[3..6];  // Top-right
    let v2 = &mesh.vertices[6..9];  // Bottom-left
    let v3 = &mesh.vertices[9..12]; // Bottom-right
    
    // Verify vertex positions
    assert_eq!(v0, [0.0, 1.0, 0.0]); // Top-left
    assert_eq!(v1, [1.0, 1.0, 0.0]); // Top-right
    assert_eq!(v2, [0.0, 1.0, 1.0]); // Bottom-left
    assert_eq!(v3, [1.0, 1.0, 1.0]); // Bottom-right
    
    // Verify triangle indices form counter-clockwise winding when viewed from above
    let indices = &mesh.indices;
    assert_eq!(indices[0..3], [0, 1, 2]); // First triangle
    assert_eq!(indices[3..6], [1, 3, 2]); // Second triangle
    
    // Verify normal is pointing up
    assert_eq!(&mesh.normals[0..3], [0.0, 1.0, 0.0]);
} 