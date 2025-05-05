use crate::terrain::{
    coords::{XZCoords, CHUNK_SIZE},
    generator::HeightmapGenerator,
};
use approx::assert_relative_eq;
use test_case::test_case;
use log::info;

#[test]
fn test_heightmap_generator_creation() {
    let generator = HeightmapGenerator::new(42);
    let coord = XZCoords { x: 0, z: 0 };
    let heights = generator.generate_chunk(coord);
    assert!(!heights.is_empty());
}

#[test]
fn test_heightmap_generation_dimensions() {
    let generator = HeightmapGenerator::new(42);
    let coord = XZCoords { x: 0, z: 0 };
    let heights = generator.generate_chunk(coord);
    
    assert_eq!(heights.len(), (CHUNK_SIZE as usize) * (CHUNK_SIZE as usize));
}

#[test]
fn test_height_range() {
    let generator = HeightmapGenerator::new(42);
    let coord = XZCoords { x: 0, z: 0 };
    let heights = generator.generate_chunk(coord);
    
    // Heights should be between -HEIGHT_RANGE and +HEIGHT_RANGE
    for height in heights {
        assert!(height >= -32.0);  // -HEIGHT_RANGE
        assert!(height <= 32.0);   // +HEIGHT_RANGE
    }
}

#[test_case(1, 0)]
#[test_case(0, 1)]
#[test_case(-1, -1)]
fn test_chunk_position_affects_heights(x: i32, z: i32) {
    let generator = HeightmapGenerator::new(42);
    let coord1 = XZCoords { x: 0, z: 0 };
    let coord2 = XZCoords { x, z };
    
    let heights1 = generator.generate_chunk(coord1);
    let heights2 = generator.generate_chunk(coord2);
    
    // Different chunk positions should generally produce different heights
    assert!(heights1 != heights2, "Heights should differ for different chunk positions");
}

#[test]
fn test_height_continuity() {
    let generator = HeightmapGenerator::new(42);
    let coord1 = XZCoords { x: 0, z: 0 };
    let coord2 = XZCoords { x: 1, z: 0 };
    
    let heights1 = generator.generate_chunk(coord1);
    let heights2 = generator.generate_chunk(coord2);
    
    // Check that heights along the border between chunks are continuous
    for z in 0..CHUNK_SIZE as usize {
        let h1 = heights1[z * CHUNK_SIZE as usize + (CHUNK_SIZE as usize - 1)];
        let h2 = heights2[z * CHUNK_SIZE as usize];
        
        // Heights should be relatively close at chunk boundaries
        assert_relative_eq!(h1, h2, epsilon = 1.0);
    }
}

#[test]
fn test_padded_heightmap() {
    let generator = HeightmapGenerator::new(42);
    let coord = XZCoords { x: 0, z: 0 };
    let padded = generator.generate_padded_heightmap(coord);
    
    // Test padded dimensions (35x35 for a 33x33 chunk)
    assert_eq!(padded.chunk_only().len(), (CHUNK_SIZE as usize + 1) * (CHUNK_SIZE as usize + 1));
    
    // Test that interior points match generate_chunk output
    let chunk = generator.generate_chunk(coord);
    let interior = padded.chunk_only();
    
    // Compare interior points
    for z in 0..CHUNK_SIZE as usize {
        for x in 0..CHUNK_SIZE as usize {
            let chunk_idx = z * CHUNK_SIZE as usize + x;
            let interior_idx = z * (CHUNK_SIZE as usize + 1) + x;
            if chunk_idx < chunk.len() && interior_idx < interior.len() {
                let c = chunk[chunk_idx];
                let i = interior[interior_idx];
                assert_relative_eq!(c, i, epsilon = 1e-5);
                if (c - i).abs() > 1e-5 {
                    info!("Mismatch at x={}, z={}: chunk={}, interior={}", x, z, c, i);
                }
            }
        }
    }
}

#[test]
fn test_seed_determinism() {
    let seed = 42;
    let generator1 = HeightmapGenerator::new(seed);
    let generator2 = HeightmapGenerator::new(seed);
    let coord = XZCoords { x: 0, z: 0 };
    
    let heights1 = generator1.generate_chunk(coord);
    let heights2 = generator2.generate_chunk(coord);
    
    assert_eq!(heights1, heights2, "Same seed should produce identical heights");
} 