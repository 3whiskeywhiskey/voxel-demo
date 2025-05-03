use crate::terrain::{
    coords::{ChunkCoords, CHUNK_SIZE},
    generator::HeightmapGenerator,
};
use approx::assert_relative_eq;
use test_case::test_case;

#[test]
fn test_heightmap_generator_creation() {
    let generator = HeightmapGenerator::new(42);
    let coord = ChunkCoords { x: 0, z: 0 };
    let heights = generator.generate_chunk(coord);
    assert!(!heights.is_empty());
}

#[test]
fn test_heightmap_generation_dimensions() {
    let generator = HeightmapGenerator::new(42);
    let coord = ChunkCoords { x: 0, z: 0 };
    let heights = generator.generate_chunk(coord);
    
    assert_eq!(heights.len(), CHUNK_SIZE * CHUNK_SIZE);
}

#[test]
fn test_height_range() {
    let generator = HeightmapGenerator::new(42);
    let coord = ChunkCoords { x: 0, z: 0 };
    let heights = generator.generate_chunk(coord);
    
    // Heights should be between 0 and HEIGHT_RANGE
    for height in heights {
        assert!(height >= 0.0);
        assert!(height <= 64.0); // HEIGHT_RANGE constant
    }
}

#[test_case(1, 0)]
#[test_case(0, 1)]
#[test_case(-1, -1)]
fn test_chunk_position_affects_heights(x: i32, z: i32) {
    let generator = HeightmapGenerator::new(42);
    let coord1 = ChunkCoords { x: 0, z: 0 };
    let coord2 = ChunkCoords { x, z };
    
    let heights1 = generator.generate_chunk(coord1);
    let heights2 = generator.generate_chunk(coord2);
    
    // Different chunk positions should generally produce different heights
    assert!(heights1 != heights2, "Heights should differ for different chunk positions");
}

#[test]
fn test_height_continuity() {
    let generator = HeightmapGenerator::new(42);
    let coord1 = ChunkCoords { x: 0, z: 0 };
    let coord2 = ChunkCoords { x: 1, z: 0 };
    
    let heights1 = generator.generate_chunk(coord1);
    let heights2 = generator.generate_chunk(coord2);
    
    // Check that heights along the border between chunks are continuous
    for z in 0..CHUNK_SIZE {
        let h1 = heights1[z * CHUNK_SIZE + (CHUNK_SIZE - 1)];
        let h2 = heights2[z * CHUNK_SIZE];
        
        // Heights should be relatively close at chunk boundaries
        assert_relative_eq!(h1, h2, epsilon = 1.0);
    }
}

#[test]
fn test_seed_determinism() {
    let seed = 42;
    let generator1 = HeightmapGenerator::new(seed);
    let generator2 = HeightmapGenerator::new(seed);
    let coord = ChunkCoords { x: 0, z: 0 };
    
    let heights1 = generator1.generate_chunk(coord);
    let heights2 = generator2.generate_chunk(coord);
    
    assert_eq!(heights1, heights2, "Same seed should produce identical heights");
} 