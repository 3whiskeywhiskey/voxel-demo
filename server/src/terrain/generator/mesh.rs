use nalgebra::Vector3;
use crate::terrain::{
    coords::{ChunkCoords, CHUNK_SIZE},
    generator::PaddedHeightmap,
};
use crate::entity::Mesh;
use log::debug;

pub struct MeshGenerator {
}

impl MeshGenerator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn generate_mesh(&self, coord: ChunkCoords, padded_heightmap: PaddedHeightmap) -> Mesh {
        let mesh = self.generate_dual_contour_mesh(coord, padded_heightmap);
        // let mesh = self.cuboid_mesh();
        mesh
    }

    // generates a mesh from a heightmap using the dual contouring algorithm
    fn generate_dual_contour_mesh(&self, coord: ChunkCoords, padded_heightmap: PaddedHeightmap) -> Mesh {
        let mut verts = Vec::new();
        let mut norms = Vec::new();
        let mut idxs = Vec::new();
        let mut mats = Vec::new();
        let mut vertex_count = 0u32;
        let dim = CHUNK_SIZE + 1;
        let mut cell_vertex_idx = vec![vec![None; dim]; dim];
        // record edge crossings for stitching
        let mut edge_east  = vec![vec![false; dim]; dim];
        let mut edge_south = vec![vec![false; dim]; dim];

        // compute chunk world offset
        // let world_offset = coord.to_world_pos(0, 0);
        // let world_x = world_offset.x as f32;
        // let world_z = world_offset.z as f32;

        // debug!("World offset: ({}, {})", world_x, world_z);

        // Debug: Print some heightmap values
        // for z in 0..5 {
        //     for x in 0..5 {
        //         debug!("Height at ({}, {}): {}", x, z, padded_heightmap.get(x, z));
        //     }
        // }

        struct Hermite {
            p: Vector3<f32>,
            n: Vector3<f32>,
        }
        
        // helper to check one edge, returns true if any crossing found
        fn sample_edge(
            a: (usize, usize, f32),
            b: (usize, usize, f32),
            map: &PaddedHeightmap,
            out: &mut Vec<Hermite>,
        ) -> bool {
            let (ax, az, height_a) = a;
            let (bx, bz, height_b) = b;
            let mut crossed = false;
            // Sample multiple points along Y to find crossings
            let sample_heights = [0.0, height_a.min(height_b), height_a.max(height_b)];
            for &y_sample in &sample_heights {
                let sdf_a = y_sample - height_a;
                let sdf_b = y_sample - height_b;
                if (sdf_a > 0.0) != (sdf_b > 0.0) {
                    let t = (sdf_a / (sdf_a - sdf_b)).clamp(0.0, 1.0);
                    let x = ax as f32 + t * (bx as f32 - ax as f32);
                    let z = az as f32 + t * (bz as f32 - az as f32);
                    let y = height_a + t * (height_b - height_a);
                    let x_floor = x.floor() as usize;
                    let z_floor = z.floor() as usize;
                    let x_ceil = (x_floor + 1).min(CHUNK_SIZE - 1);
                    let z_ceil = (z_floor + 1).min(CHUNK_SIZE - 1);
                    let h00 = map.get(x_floor, z_floor);
                    let h10 = map.get(x_ceil, z_floor);
                    let h01 = map.get(x_floor, z_ceil);
                    let h11 = map.get(x_ceil, z_ceil);
                    let fx = x - x_floor as f32;
                    let fz = z - z_floor as f32;
                    let dx = (h10 - h00) * (1.0 - fz) + (h11 - h01) * fz;
                    let dz = (h01 - h00) * (1.0 - fx) + (h11 - h10) * fx;
                    let normal = Vector3::new(-dx, 1.0, -dz).normalize();
                    let p = Vector3::new(x, y, z);
                    out.push(Hermite { p, n: normal });
                    crossed = true;
                }
            }
            crossed
        }
        
        // Process each cell and record crossings via sample_edge
        for z in 0..=CHUNK_SIZE {
            for x in 0..=CHUNK_SIZE {
                let mut hermites = Vec::with_capacity(4);
                let f00 = padded_heightmap.get(x, z);
                let f10 = padded_heightmap.get(x + 1, z);
                let f01 = padded_heightmap.get(x, z + 1);
                let f11 = padded_heightmap.get(x + 1, z + 1);

                // record crossings based on sample_edge result
                let east_cross  = sample_edge((x, z, f00),    (x + 1, z, f10),    &padded_heightmap, &mut hermites);
                let south_cross = sample_edge((x, z + 1, f01), (x, z, f00),       &padded_heightmap, &mut hermites);

                // still sample the remaining edges to fill hermites
                sample_edge((x + 1, z, f10),    (x + 1, z + 1, f11), &padded_heightmap, &mut hermites);
                sample_edge((x + 1, z + 1, f11), (x, z + 1, f01),    &padded_heightmap, &mut hermites);

                edge_east[z][x]  = east_cross;
                edge_south[z][x] = south_cross;

                if hermites.is_empty() { continue; }
                // Average the positions and normals
                let mut avg_pos = Vector3::zeros();
                let mut avg_norm = Vector3::zeros();
                for h in &hermites {
                    avg_pos += h.p;
                    avg_norm += h.n;
                }
                avg_pos /= hermites.len() as f32;
                avg_norm = avg_norm.normalize();
                // Store the vertex
                let idx = vertex_count;
                vertex_count += 1;
                verts.extend_from_slice(&[avg_pos.x, avg_pos.y, avg_pos.z]);
                norms.extend_from_slice(&[avg_norm.x, avg_norm.y, avg_norm.z]);
                mats.push(0);
                cell_vertex_idx[z][x] = Some(idx);
            }
        }
        
        // PASS 2: per-edge stitching
        // East edges: stitch quads between (z,x) and (z,x+1)
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                if edge_east[z][x] {
                    if let (Some(v00), Some(v10), Some(v11), Some(v01)) = (
                        cell_vertex_idx[z][x],
                        cell_vertex_idx[z + 1][x],
                        cell_vertex_idx[z + 1][x + 1],
                        cell_vertex_idx[z][x + 1],
                    ) {
                        idxs.extend_from_slice(&[v00, v10, v11]);
                        idxs.extend_from_slice(&[v00, v11, v01]);
                    }
                }
            }
        }
        // South edges: stitch quads between (z,x) and (z+1,x)
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                if edge_south[z][x] {
                    if let (Some(v00), Some(v01), Some(v11), Some(v10)) = (
                        cell_vertex_idx[z][x],
                        cell_vertex_idx[z + 1][x],
                        cell_vertex_idx[z + 1][x + 1],
                        cell_vertex_idx[z][x + 1],
                    ) {
                        idxs.extend_from_slice(&[v00, v01, v11]);
                        idxs.extend_from_slice(&[v00, v11, v10]);
                    }
                }
            }
        }
        
        // debug!("Generated {} vertices, {} indices", verts.len() / 3, idxs.len());
        
        // Create the final mesh
        Mesh {
            id: 0,
            vertices: verts,
            normals: norms,
            indices: idxs,
            materials: mats,
        }
    }

    // Legacy method for blocky mesh generation - kept for testing
    pub fn heightmap_to_blocky_mesh(&self, coord: ChunkCoords, heights: Vec<f32>) -> Mesh {
        let mut verts: Vec<f32> = Vec::new();
        let mut norms: Vec<f32> = Vec::new();
        let mut idxs: Vec<u32> = Vec::new();
        let mut mats: Vec<u32> = Vec::new();

        // compute chunk world offset
        let world_offset = coord.to_world_pos(0, 0);
        let _world_x = world_offset.x;
        let _world_z = world_offset.z;

        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let h = heights[(z * CHUNK_SIZE + x) as usize];
                if h <= 0.0 { continue; }

                // fetch neighbor heights (treat out‑of‑bounds as h)
                let north = if z + 1 < CHUNK_SIZE { heights[((z + 1) * CHUNK_SIZE + x) as usize] } else { h };
                let south = if z > 0 { heights[((z - 1) * CHUNK_SIZE + x) as usize] } else { h };
                let east  = if x + 1 < CHUNK_SIZE { heights[(z * CHUNK_SIZE + (x + 1)) as usize] } else { h };
                let west  = if x > 0 { heights[(z * CHUNK_SIZE + (x - 1)) as usize] } else { h };
                
                // Four corners of top quad
                let base = (verts.len() / 3) as u32;
                // Use local coordinates (x, z within the chunk)
                let local_x = x as f32;
                let local_z = z as f32;
                
                // Add vertices with correct height (relative to chunk origin)
                let y = h; // Use height directly for Y coordinate
                
                // Top face vertices (local coords)
                verts.extend([local_x, y, local_z]);           // Top-left (local)
                verts.extend([local_x + 1.0, y, local_z]);     // Top-right (local)
                verts.extend([local_x, y, local_z + 1.0]);     // Bottom-left (local)
                verts.extend([local_x + 1.0, y, local_z + 1.0]); // Bottom-right (local)
                
                // Push four copies of the upward normal
                norms.extend(std::iter::repeat([0.0f32, 1.0f32, 0.0f32]).take(4).flatten());
                
                // One material per-vertex
                mats.extend([0; 4]); // e.g. grass=0
                
                // Top face (counter-clockwise winding from above)
                idxs.extend([base, base+2, base+1, base+1, base+2, base+3]);

                // Side and bottom faces
                // Front (+Z)
                if north < h {
                    let side_base = (verts.len() / 3) as u32;
                    verts.extend([local_x,    0.0, local_z + 1.0]);
                    verts.extend([local_x + 1.0, 0.0, local_z + 1.0]);
                    verts.extend([local_x,       y, local_z + 1.0]);
                    verts.extend([local_x + 1.0,   y, local_z + 1.0]);
                    norms.extend(std::iter::repeat([0.0f32, 0.0f32, 1.0f32]).take(4).flatten());
                    mats.extend([0; 4]);
                    idxs.extend([side_base, side_base + 2, side_base + 1, side_base + 1, side_base + 2, side_base + 3]);
                }

                // Back (-Z)
                if south < h {
                    let side_base = (verts.len() / 3) as u32;
                    verts.extend([local_x + 1.0, 0.0, local_z]);
                    verts.extend([local_x,       0.0, local_z]);
                    verts.extend([local_x + 1.0,   y, local_z]);
                    verts.extend([local_x,         y, local_z]);
                    norms.extend(std::iter::repeat([0.0f32, 0.0f32, -1.0f32]).take(4).flatten());
                    mats.extend([0; 4]);
                    idxs.extend([side_base, side_base + 2, side_base + 1, side_base + 1, side_base + 2, side_base + 3]);
                }

                // Right (+X)
                if east < h {
                    let side_base = (verts.len() / 3) as u32;
                    verts.extend([local_x + 1.0, 0.0, local_z + 1.0]);
                    verts.extend([local_x + 1.0, 0.0, local_z]);
                    verts.extend([local_x + 1.0,   y, local_z + 1.0]);
                    verts.extend([local_x + 1.0,   y, local_z]);
                    norms.extend(std::iter::repeat([1.0f32, 0.0f32, 0.0f32]).take(4).flatten());
                    mats.extend([0; 4]);
                    idxs.extend([side_base, side_base + 2, side_base + 1, side_base + 1, side_base + 2, side_base + 3]);
                }

                // Left (-X)
                if west < h {
                    let side_base = (verts.len() / 3) as u32;
                    verts.extend([local_x,    0.0, local_z]);
                    verts.extend([local_x,    0.0, local_z + 1.0]);
                    verts.extend([local_x,      y, local_z]);
                    verts.extend([local_x,      y, local_z + 1.0]);
                    norms.extend(std::iter::repeat([-1.0f32, 0.0f32, 0.0f32]).take(4).flatten());
                    mats.extend([0; 4]);
                    idxs.extend([side_base, side_base + 2, side_base + 1, side_base + 1, side_base + 2, side_base + 3]);
                }

                // Bottom face (y = 0)
                // Skipped to avoid under-chunk geometry
            }
        }
        
        Mesh {
            id: 0,
            vertices: verts,
            normals: norms,
            indices: idxs,
            materials: mats,
        }
    }
}
