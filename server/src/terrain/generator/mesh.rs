use nalgebra::{Matrix3, Vector3};
use crate::{
    terrain::{
        coords::{ChunkCoords, CHUNK_SIZE, Vec3},
        generator::PaddedHeightmap,
    },
    entity::Mesh,
};

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
        let mut verts: Vec<f32> = Vec::new();
        let mut norms: Vec<f32> = Vec::new();
        let mut idxs: Vec<u32> = Vec::new();
        let mut mats: Vec<u32> = Vec::new();

        // compute chunk world offset
        let world_offset = coord.to_world_pos(0, 0);
        let world_x = world_offset.x;
        let world_z = world_offset.z;

        struct Hermite {
            p: Vector3<f32>,
            n: Vector3<f32>,
        }
        
        // helper to check one edge
        fn sample_edge(
            a: (usize, usize, f32),
            b: (usize, usize, f32),
            world_x: f32,
            world_z: f32,
            map: &PaddedHeightmap,
            out: &mut Vec<Hermite>,
        ) {
            let (ax, az, fa) = a;
            let (bx, bz, fb) = b;
            if (fa > 0.0) != (fb > 0.0) {
                // param t for zero‐crossing
                let t = fa / (fa - fb);

                let px = world_x + (ax as f32 + t * ((bx-ax) as f32));
                let pz = world_z + (az as f32 + t * ((bz-az) as f32));
                let py = 0.0; // for heightfields, y=0 is isosurface

                // approximate normal by central‐diff in the heightmap
                let dx = map.get(ax+1, az) - map.get(ax-1, az);
                let dz = map.get(ax,   az+1) - map.get(ax,   az-1);

                let normal = Vector3::new(-dx, 2.0, -dz).normalize();
                
                out.push(Hermite { p: Vector3::new(px, py, pz), n: normal });
            }
        }
        
        let mut hermites: Vec<Hermite> = Vec::with_capacity(4);
        let mut cell_vertex_idx: Vec<Vec<Option<u32>>> = vec![vec![None; CHUNK_SIZE]; CHUNK_SIZE];

        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                // corner heights
                let f00 = padded_heightmap.get(x  , z  );
                let f10 = padded_heightmap.get(x+1, z  );
                let f01 = padded_heightmap.get(x  , z+1);
                let f11 = padded_heightmap.get(x+1, z+1);

                hermites.clear();

                // sample the edges to find zero crossings, store them in hermites
                sample_edge((x,  z,  f00), (x+1, z,  f10), world_x, world_z, &padded_heightmap, &mut hermites);
                sample_edge((x+1,z,  f10), (x+1, z+1,f11), world_x, world_z, &padded_heightmap, &mut hermites);
                sample_edge((x+1,z+1,f11), (x,   z+1,f01), world_x, world_z, &padded_heightmap, &mut hermites);
                sample_edge((x,  z+1,f01), (x,   z,  f00), world_x, world_z, &padded_heightmap, &mut hermites);

                // if there was at least one crossing, perform QEF 
                if !hermites.is_empty() {
                    // build ATA and ATb from hermites[..]
                    let mut ata = Matrix3::zeros();
                    let mut atb = Vector3::zeros();
                    for h in &hermites {
                        // Outer product: n * n^T
                        ata += h.n * h.n.transpose();
                        // Dot expects a reference
                        atb += h.n * h.n.dot(&h.p);
                    }
                    // solve for v
                    let mut v = ata.try_inverse().unwrap_or(Matrix3::identity()) * atb;

                    // optionally clamp v back into the [x..x+1]×[z..z+1] cell bounds
                    v.x = v.x.clamp(world_x + x as f32, world_x + x as f32 + 1.0);
                    v.z = v.z.clamp(world_z + z as f32, world_z + z as f32 + 1.0);

                    // 4) emit that vertex and remember its index
                    let idx = (verts.len() / 3) as u32;
                    verts.extend_from_slice(&[v.x, v.y, v.z]);
                    // compute vertex normal by averaging Hermite normals
                    let normal: Vector3<f32> = hermites.iter()
                        .fold(Vector3::zeros(), |sum, h| sum + h.n)
                        .normalize();
                    norms.extend_from_slice(&[normal.x, normal.y, normal.z]);
                    // placeholder material ID per vertex
                    mats.push(0);
                    cell_vertex_idx[z as usize][x as usize] = Some(idx);
                }
            }
        }
        
        // emit the triangles
        for z in 0..CHUNK_SIZE-1 {
            for x in 0..CHUNK_SIZE-1 {
                let v00 = match cell_vertex_idx[z  ][x  ] { Some(i) => i, None => continue };
                let v01 = match cell_vertex_idx[z  ][x+1] { Some(i) => i, None => continue };
                let v10 = match cell_vertex_idx[z+1][x  ] { Some(i) => i, None => continue };
                let v11 = match cell_vertex_idx[z+1][x+1] { Some(i) => i, None => continue };

                // idxs.extend([v00, v01, v10, v01, v11, v10]);
                idxs.extend([v00, v10, v01, v10, v11, v01]);
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


    fn heightmap_to_blocky_mesh(&self, coord: ChunkCoords, heights: Vec<f32>) -> Mesh {
        let mut verts: Vec<f32> = Vec::new();
        let mut norms: Vec<f32> = Vec::new();
        let mut idxs: Vec<u32> = Vec::new();
        let mut mats: Vec<u32> = Vec::new();

        // compute chunk world offset

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
