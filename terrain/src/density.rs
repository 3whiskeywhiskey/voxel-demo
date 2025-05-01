// src/density.rs

// use spacetimedb::{table, reducer};
// use crate::coords::{ChunkCoords, CHUNK_SIZE, MaterialId};

// #[cfg_attr(feature="server", table(name="density_chunk"))]
// #[cfg_attr(feature="server", derive(Clone, Debug))]
// pub struct DensityChunk {
//     #[cfg_attr(feature="server", spacetimedb(primary_key))]
//     pub coord: ChunkCoords,
//     pub densities: Vec<f32>,    // (CHUNK_SIZE+1)^3 samples
//     pub materials: Vec<MaterialId>,
// }

// pub struct DensityUpdated {
//     pub coord: ChunkCoords,
//     pub deltas: Vec<(usize, f32)>,
// }
