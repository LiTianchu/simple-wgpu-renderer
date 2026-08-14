use std::mem;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pos_x: f32,
    pos_y: f32,
    pos_z: f32,
    uv_x: f32,
    uv_y: f32,
    norm_x: f32,
    norm_y: f32,
    norm_z: f32,
}

impl Vertex {
    pub fn new() -> Self {
        Self {
            pos_x: Default::default(),
            pos_y: Default::default(),
            pos_z: Default::default(),
            uv_x: Default::default(),
            uv_y: Default::default(),
            norm_x: Default::default(),
            norm_y: Default::default(),
            norm_z: Default::default(),
        }
    }

    pub fn with_position(mut self, pos_x: f32, pos_y: f32, pos_z: f32) -> Self {
        self.pos_x = pos_x;
        self.pos_y = pos_y;
        self.pos_z = pos_z;
        self
    }

    pub fn with_uv(mut self, uv_x: f32, uv_y: f32) -> Self {
        self.uv_x = uv_x;
        self.uv_y = uv_y;
        self
    }

    pub fn with_normal(mut self, norm_x: f32, norm_y: f32, norm_z: f32) -> Self {
        self.norm_x = norm_x;
        self.norm_y = norm_y;
        self.norm_z = norm_z;
        self
    }

    pub const BUFFER_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![
            0=>Float32x3,
            1=>Float32x2,
            2=>Float32x3
        ],
    };
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Face {
    index_1: u32,
    index_2: u32,
    index_3: u32,
}

impl Face {
    pub fn new(index_1: u32, index_2: u32, index_3: u32) -> Self {
        Self {
            index_1,
            index_2,
            index_3,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialAttributeSet {
    pub k_ambient: [f32; 3],
    pub k_diffuse: [f32; 3],
    pub k_specular: [f32; 3],
    pub k_emissive: [f32; 3],
    pub index_of_refraction: f32,
    pub shininess: f32,
    pub dissolve: f32,
    pub illumination_model: u32,
}

impl Default for MaterialAttributeSet {
    fn default() -> Self {
        // Blender standard defaults
        Self {
            k_ambient: [1.0, 1.0, 1.0],
            k_diffuse: [0.8, 0.8, 0.8],
            k_specular: [0.5, 0.5, 0.5],
            k_emissive: [0.0, 0.0, 0.0],
            index_of_refraction: 1.45,
            shininess: 225.0,
            dissolve: 1.0,
            illumination_model: 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextureSet {
    pub normal_map_path: Option<String>,
    pub ambient_map_path: Option<String>,
    pub diffuse_map_path: Option<String>,
    pub specular_map_path: Option<String>,
    pub shininess_map_path: Option<String>,
    pub dissolve_map_path: Option<String>,
}

impl TextureSet {
    pub fn new() -> Self {
        Self {
            normal_map_path: None,
            ambient_map_path: None,
            diffuse_map_path: None,
            specular_map_path: None,
            shininess_map_path: None,
            dissolve_map_path: None,
        }
    }

    pub fn with_normal(mut self, normal_map_path: impl Into<String>) -> Self {
        self.normal_map_path = Some(normal_map_path.into());
        self
    }

    pub fn with_ambient(mut self, ambient_map_path: impl Into<String>) -> Self {
        self.ambient_map_path = Some(ambient_map_path.into());
        self
    }
    pub fn with_diffuse(mut self, diffuse_map_path: impl Into<String>) -> Self {
        self.diffuse_map_path = Some(diffuse_map_path.into());
        self
    }
    pub fn with_specular(mut self, specular_map_path: impl Into<String>) -> Self {
        self.specular_map_path = Some(specular_map_path.into());
        self
    }
    pub fn with_shininess(mut self, shininess_map_path: impl Into<String>) -> Self {
        self.shininess_map_path = Some(shininess_map_path.into());
        self
    }
    pub fn with_dissolve(mut self, dissolve_map_path: impl Into<String>) -> Self {
        self.dissolve_map_path = Some(dissolve_map_path.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct Material {
    pub mat_attr: MaterialAttributeSet,
    pub texture_set: TextureSet,
}

impl Material {
    pub fn new() -> Self {
        Self {
            mat_attr: MaterialAttributeSet::default(),
            texture_set: TextureSet::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mesh {
    verts: Vec<Vertex>,
    faces: Vec<Face>,
    mat_id: usize,
}

impl Mesh {
    pub fn new() -> Self {
        Self {
            verts: Vec::new(),
            faces: Vec::new(),
            mat_id: 0,
        }
    }
    pub fn verts(&self) -> &[Vertex] {
        &self.verts
    }

    pub fn faces(&self) -> &[Face] {
        &self.faces
    }

    pub fn push_vert(&mut self, vert: Vertex) {
        self.verts.push(vert);
    }

    pub fn push_face(&mut self, index_1: u32, index_2: u32, index_3: u32) {
        self.faces.push(Face {
            index_1,
            index_2,
            index_3,
        })
    }

    pub fn set_mat_id(&mut self, new_mat_id: usize) {
        self.mat_id = new_mat_id;
    }

    pub fn with_mat_id(mut self, mat_id: usize) -> Self {
        self.mat_id = mat_id;
        self
    }
}

#[derive(Debug, Clone)]
pub struct Model {
    meshes: Vec<Mesh>,
    materials: Vec<Material>,
}

impl Model {
    pub fn new() -> Self {
        Self {
            meshes: Vec::new(),
            materials: Vec::new(),
        }
    }

    pub fn meshes(&self) -> &[Mesh] {
        &self.meshes
    }

    pub fn materials(&self) -> &[Material] {
        &self.materials
    }

    pub fn mesh_mut(&mut self) -> &mut [Mesh] {
        &mut self.meshes
    }

    pub fn push_mesh(&mut self, mesh: Mesh) {
        self.meshes.push(mesh);
    }

    pub fn push_material(&mut self, material: Material) {
        self.materials.push(material);
    }

    pub fn all_faces_iter(&self) -> impl Iterator<Item = &Face> {
        self.meshes.iter().flat_map(|mesh| mesh.faces())
    }

    pub fn all_faces_ref(&self) -> Vec<&Face> {
        self.meshes
            .iter()
            .flat_map(|mesh| mesh.faces())
            .collect::<Vec<_>>()
    }

    // flattens all faces across meshes, offsetting each mesh's local
    // vertex indices by the cumulative vertex count of prior meshes
    // if not the indices will not be correct
    pub fn all_faces_copied(&self) -> Vec<Face> {
        let mut result = Vec::with_capacity(self.face_count());
        let mut vertex_offset: u32 = 0;

        for mesh in &self.meshes {
            for face in mesh.faces() {
                result.push(Face::new(
                    face.index_1 + vertex_offset,
                    face.index_2 + vertex_offset,
                    face.index_3 + vertex_offset,
                ));
            }
            vertex_offset += mesh.verts().len() as u32;
        }

        result
    }

    pub fn all_verts_iter(&self) -> impl Iterator<Item = &Vertex> {
        self.meshes.iter().flat_map(|mesh| mesh.verts())
    }

    pub fn all_verts_ref(&self) -> Vec<&Vertex> {
        self.meshes
            .iter()
            .flat_map(|mesh| mesh.verts())
            .collect::<Vec<_>>()
    }

    pub fn all_verts_copied(&self) -> Vec<Vertex> {
        self.meshes
            .iter()
            .flat_map(|mesh| mesh.verts().iter().copied())
            .collect::<Vec<_>>()
    }

    pub fn face_count(&self) -> usize {
        self.meshes.iter().map(|mesh| mesh.faces().len()).sum()
    }

    pub fn vert_count(&self) -> usize {
        self.meshes.iter().map(|mesh| mesh.verts().len()).sum()
    }
}
