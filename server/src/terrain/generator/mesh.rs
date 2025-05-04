use nalgebra::Vector3;
use nalgebra::Matrix3;
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

        // helper to sample the vertical edge at (x,z) for heightmap-based terrain
        fn sample_vert_edge(
            x0: usize,
            z0: usize,
            map: &PaddedHeightmap,
            out: &mut Vec<Hermite>,
        ) {
            let h = map.get(x0, z0);
            // position on the surface
            let p = Vector3::new(x0 as f32, h, z0 as f32);
            // approximate normal from height gradients
            let dx = map.get(x0+1, z0) - map.get(x0.saturating_sub(1), z0);
            let dz = map.get(x0, z0+1) - map.get(x0, z0.saturating_sub(1));
            let n = Vector3::new(-dx, 2.0, -dz).normalize();
            out.push(Hermite { p, n });
        }

        struct Hermite {
            p: Vector3<f32>,
            n: Vector3<f32>,
        }
        
        // Process each cell and record crossings via sample_vert_edge
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let mut hermites = Vec::with_capacity(4);
                // sample the four vertical edges at the corners
                sample_vert_edge(x,   z,   &padded_heightmap, &mut hermites);
                sample_vert_edge(x+1, z,   &padded_heightmap, &mut hermites);
                sample_vert_edge(x+1, z+1, &padded_heightmap, &mut hermites);
                sample_vert_edge(x,   z+1, &padded_heightmap, &mut hermites);

                // record crossings for stitching
                let f00 = padded_heightmap.get(x, z);
                let f10 = padded_heightmap.get(x+1, z);
                let f01 = padded_heightmap.get(x, z+1);
                let f11 = padded_heightmap.get(x+1, z+1);

                // QEF solve: ATA v = ATb
                let mut ata = Matrix3::zeros();
                let mut atb = Vector3::zeros();
                for h in &hermites {
                    ata += h.n * h.n.transpose();
                    atb += h.n * h.n.dot(&h.p);
                }
                let mut v = ata.try_inverse().unwrap_or(Matrix3::identity()) * atb;
                // clamp into cell bounds
                let min_x = x as f32;
                let min_z = z as f32;
                v.x = v.x.clamp(min_x,     min_x + 1.0);
                v.z = v.z.clamp(min_z,     min_z + 1.0);
                // clamp y into height range of cell
                let min_h = f00.min(f10).min(f01).min(f11);
                let max_h = f00.max(f10).max(f01).max(f11);
                v.y = v.y.clamp(min_h, max_h);

                // emit vertex
                let idx = vertex_count;
                vertex_count += 1;
                verts.extend_from_slice(&[v.x, v.y, v.z]);
                // normal for shading
                let normal = hermites.iter().fold(Vector3::zeros(), |s,h| s + h.n).normalize();
                norms.extend_from_slice(&[normal.x, normal.y, normal.z]);
                mats.push(0);
                cell_vertex_idx[z][x] = Some(idx);
            }
        }
        
        // PASS 2: unconditional stitching of all cells
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                if let (Some(v00), Some(v10), Some(v11), Some(v01)) = (
                    cell_vertex_idx[z][x],
                    cell_vertex_idx[z][x + 1],
                    cell_vertex_idx[z + 1][x + 1],
                    cell_vertex_idx[z + 1][x],
                ) {
                    // Flip winding: triangle facing up when viewed from above
                    idxs.extend_from_slice(&[v00, v11, v10]);
                    idxs.extend_from_slice(&[v00, v01, v11]);
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
