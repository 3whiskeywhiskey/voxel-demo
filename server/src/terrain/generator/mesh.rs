use crate::{
    terrain::{
        Chunk,
        coords::{ChunkCoords, CHUNK_SIZE},
    },
    entity::Mesh,
};

pub struct MeshGenerator {
}

impl MeshGenerator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn generate_mesh(&self, coord: ChunkCoords, heights: Vec<f32>) -> Mesh {
        let mesh = self.heightmap_to_blocky_mesh(coord, heights);
        mesh
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
                // Four corners of top quad
                let base = (verts.len() / 3) as u32;
                let pos = coord.to_world_pos(x, z);
                verts.extend([pos.x, pos.y + h, pos.z]);
                verts.extend([pos.x + 1.0, pos.y + h, pos.z]);
                verts.extend([pos.x, pos.y + h, pos.z + 1.0]);
                verts.extend([pos.x + 1.0, pos.y + h, pos.z + 1.0]);
                // Push four copies of the normal without requiring Vec3: Copy
                norms.extend(std::iter::repeat([0.0f32, 1.0f32, 0.0f32]).take(4).flatten());
                // One material per-vertex
                mats.extend([0; 4]); // e.g. grass=0
                // Two triangles
                idxs.extend([base, base+2, base+1, base+2, base+3, base+1]);
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
