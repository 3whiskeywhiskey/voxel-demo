use crate::{
    terrain::coords::{ChunkCoords, CHUNK_SIZE},
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
        // let mesh = self.cuboid_mesh();
        mesh
    }

    /*
     Cuboid mesh: Mesh { primitive_topology: TriangleList, 
     attributes: {
     MeshVertexAttributeId(0): MeshAttributeData { attribute: MeshVertexAttribute { name: "Vertex_Position", id: MeshVertexAttributeId(0), format: Float32x3 }, values: Float32x3([[-0.5, -0.5, 0.5], [0.5, -0.5, 0.5], [0.5, 0.5, 0.5], [-0.5, 0.5, 0.5], [-0.5, 0.5, -0.5], [0.5, 0.5, -0.5], [0.5, -0.5, -0.5], [-0.5, -0.5, -0.5], [0.5, -0.5, -0.5], [0.5, 0.5, -0.5], [0.5, 0.5, 0.5], [0.5, -0.5, 0.5], [-0.5, -0.5, 0.5], [-0.5, 0.5, 0.5], [-0.5, 0.5, -0.5], [-0.5, -0.5, -0.5], [0.5, 0.5, -0.5], [-0.5, 0.5, -0.5], [-0.5, 0.5, 0.5], [0.5, 0.5, 0.5], [0.5, -0.5, 0.5], [-0.5, -0.5, 0.5], [-0.5, -0.5, -0.5], [0.5, -0.5, -0.5]]) }, 
     MeshVertexAttributeId(1): MeshAttributeData { attribute: MeshVertexAttribute { name: "Vertex_Normal", id: MeshVertexAttributeId(1), format: Float32x3 }, values: Float32x3([[0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, -1.0, 0.0], [0.0, -1.0, 0.0], [0.0, -1.0, 0.0], [0.0, -1.0, 0.0]]) }, 
     MeshVertexAttributeId(2): MeshAttributeData { attribute: MeshVertexAttribute { name: "Vertex_Uv", id: MeshVertexAttributeId(2), format: Float32x2 }, values: Float32x2([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [1.0, 0.0], [0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [1.0, 0.0], [0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]) }}, indices: Some(U32([0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4, 8, 9, 10, 10, 11, 8, 12, 13, 14, 14, 15, 12, 16, 17, 18, 18, 19, 16, 20, 21, 22, 22, 23, 20])), morph_targets: None, morph_target_names: None, asset_usage: RenderAssetUsages(MAIN_WORLD | RENDER_WORLD) }

     we need to generate the equivalent Mesh
     */
    fn cuboid_mesh(&self) -> Mesh {
        let mut verts: Vec<f32> = Vec::new();
        let mut norms: Vec<f32> = Vec::new();
        let mut idxs: Vec<u32> = Vec::new();
        let mut mats: Vec<u32> = Vec::new();

        // Front face
        verts.extend([-0.5, -0.5, 0.5]);  // 0
        verts.extend([0.5, -0.5, 0.5]);   // 1
        verts.extend([0.5, 0.5, 0.5]);    // 2
        verts.extend([-0.5, 0.5, 0.5]);   // 3

        // Back face
        verts.extend([-0.5, 0.5, -0.5]);  // 4
        verts.extend([0.5, 0.5, -0.5]);   // 5
        verts.extend([0.5, -0.5, -0.5]);  // 6
        verts.extend([-0.5, -0.5, -0.5]); // 7

        // Right face
        verts.extend([0.5, -0.5, -0.5]);  // 8
        verts.extend([0.5, 0.5, -0.5]);   // 9
        verts.extend([0.5, 0.5, 0.5]);    // 10
        verts.extend([0.5, -0.5, 0.5]);   // 11

        // Left face
        verts.extend([-0.5, -0.5, 0.5]);  // 12
        verts.extend([-0.5, 0.5, 0.5]);   // 13
        verts.extend([-0.5, 0.5, -0.5]);  // 14
        verts.extend([-0.5, -0.5, -0.5]); // 15

        // Top face
        verts.extend([0.5, 0.5, -0.5]);   // 16
        verts.extend([-0.5, 0.5, -0.5]);  // 17
        verts.extend([-0.5, 0.5, 0.5]);   // 18
        verts.extend([0.5, 0.5, 0.5]);    // 19

        // Bottom face
        verts.extend([0.5, -0.5, 0.5]);   // 20
        verts.extend([-0.5, -0.5, 0.5]);  // 21
        verts.extend([-0.5, -0.5, -0.5]); // 22
        verts.extend([0.5, -0.5, -0.5]);  // 23

        // Normals for each face (4 vertices per face)
        for _ in 0..4 { norms.extend([0.0, 0.0, 1.0]); }   // Front
        for _ in 0..4 { norms.extend([0.0, 0.0, -1.0]); }  // Back
        for _ in 0..4 { norms.extend([1.0, 0.0, 0.0]); }   // Right
        for _ in 0..4 { norms.extend([-1.0, 0.0, 0.0]); }  // Left
        for _ in 0..4 { norms.extend([0.0, 1.0, 0.0]); }   // Top
        for _ in 0..4 { norms.extend([0.0, -1.0, 0.0]); }  // Bottom

        // Add material indices (one per vertex)
        mats.extend(vec![0; 24]); // 24 vertices total

        // Indices for each face (2 triangles per face)
        idxs.extend([0, 1, 2, 2, 3, 0]);     // Front
        idxs.extend([4, 5, 6, 6, 7, 4]);     // Back
        idxs.extend([8, 9, 10, 10, 11, 8]);  // Right
        idxs.extend([12, 13, 14, 14, 15, 12]); // Left
        idxs.extend([16, 17, 18, 18, 19, 16]); // Top
        idxs.extend([20, 21, 22, 22, 23, 20]); // Bottom

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
                
                // Two triangles (counter-clockwise winding)
                idxs.extend([base, base+1, base+2, base+1, base+3, base+2]);
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
