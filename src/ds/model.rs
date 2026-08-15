use crate::io::file_op;
use std::collections::HashMap;
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
    mat_key: Option<String>,
}

impl Mesh {
    pub fn new() -> Self {
        Self {
            verts: Vec::new(),
            faces: Vec::new(),
            mat_key: None,
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

    pub fn set_mat_key(&mut self, mat_key: String) {
        self.mat_key = Some(mat_key);
    }

    pub fn with_mat_key(mut self, mat_key: String) -> Self {
        self.mat_key = Some(mat_key);
        self
    }

    pub fn has_material(&self) -> bool {
        self.mat_key.is_some()
    }

    pub fn mat_key(&self) -> Option<&String> {
        self.mat_key.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct TextureObject {
    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

#[derive(Debug, Clone)]
pub struct TextureStore {
    textures: HashMap<String, TextureObject>, // file_path -> texture
}

impl TextureStore {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
        }
    }

    pub fn textures(&self) -> &HashMap<String, TextureObject> {
        &self.textures
    }

    pub fn insert_texture(&mut self, texture_key: String, texture: TextureObject) {
        self.textures.insert(texture_key, texture);
    }

    pub fn get_texture(&self, texture_key: impl Into<String>) -> Option<&TextureObject> {
        self.textures.get(&texture_key.into())
    }

    pub fn get_texture_mut(
        &mut self,
        texture_key: impl Into<String>,
    ) -> Option<&mut TextureObject> {
        self.textures.get_mut(&texture_key.into())
    }

    pub fn get_or_load_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_file_path: impl Into<String>,
        texture_format: wgpu::TextureFormat,
        texture_label: impl Into<String>,
    ) -> Option<&TextureObject> {
        let texture_file_path_str = texture_file_path.into();

        if self.get_texture(&texture_file_path_str).is_some() {
            return self.get_texture(texture_file_path_str);
        }

        let texture_label = texture_label.into();
        let data = file_op::load_binary(&texture_file_path_str).ok()?;

        let texture_file_ext = std::path::Path::new(&texture_file_path_str)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        let image_result = if texture_file_ext == "tga" {
            image::load_from_memory_with_format(&data, image::ImageFormat::Tga)
        } else {
            image::load_from_memory(&data)
        };

        if let Ok(texture_img) = image_result {
            let texture_img_rgba = texture_img.to_rgba8();
            let dimensions = texture_img_rgba.dimensions();
            let texture_size = wgpu::Extent3d {
                width: dimensions.0,
                height: dimensions.1,
                depth_or_array_layers: 1,
            };
            let texture_descriptor = wgpu::TextureDescriptor {
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: texture_format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::COPY_DST,
                label: Some(&texture_label),
                view_formats: &[],
            };
            let wgpu_texture = device.create_texture(&texture_descriptor);

            // NOTE: write_texture does not require byte alignment
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &wgpu_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &texture_img_rgba,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * dimensions.0),
                    rows_per_image: Some(dimensions.1),
                },
                texture_size,
            );
            let texture_view = wgpu_texture.create_view(&wgpu::TextureViewDescriptor::default());

            // NOTE: there are some model's UV is more than 1.0 meant for repeating
            let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some(&format!("{} Sampler", &texture_label)),
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::Repeat,
                mag_filter: wgpu::FilterMode::Linear, // pixel art
                min_filter: wgpu::FilterMode::Linear, // pixel art
                mipmap_filter: wgpu::MipmapFilterMode::Linear, // pixel art
                ..Default::default()
            });
            self.insert_texture(
                texture_file_path_str.to_string(),
                TextureObject {
                    texture: wgpu_texture,
                    texture_view,
                    sampler,
                },
            );

            return self.get_texture(texture_file_path_str);
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct MaterialStore {
    materials: HashMap<String, Material>, // file_path -> material
}

impl MaterialStore {
    pub fn new() -> Self {
        Self {
            materials: HashMap::new(),
        }
    }

    pub fn materials(&self) -> &HashMap<String, Material> {
        &self.materials
    }

    pub fn insert_material(&mut self, material_subpath: String, material: Material) {
        self.materials.insert(material_subpath, material);
    }

    pub fn get_material(&self, material_key: impl Into<String>) -> Option<&Material> {
        self.materials.get(&material_key.into())
    }

    pub fn get_material_mut(&mut self, material_key: impl Into<String>) -> Option<&mut Material> {
        self.materials.get_mut(&material_key.into())
    }
}

#[derive(Debug, Clone)]
pub struct Model {
    meshes: Vec<Mesh>,
    model_dir_path: String,
    model_filename: String,
}

impl Model {
    pub fn new() -> Self {
        Self {
            meshes: Vec::new(),
            model_dir_path: String::new(),
            model_filename: String::new(),
        }
    }

    pub fn meshes(&self) -> &[Mesh] {
        &self.meshes
    }

    pub fn mesh_mut(&mut self) -> &mut [Mesh] {
        &mut self.meshes
    }

    pub fn push_mesh(&mut self, mesh: Mesh) {
        self.meshes.push(mesh);
    }

    pub fn set_model_dir_path(&mut self, model_dir_path: String) {
        self.model_dir_path = model_dir_path;
    }

    pub fn set_model_filename(&mut self, model_filename: String) {
        self.model_filename = model_filename;
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

    pub fn model_dir_path(&self) -> &str {
        &self.model_dir_path
    }

    pub fn model_filename(&self) -> &str {
        &self.model_filename
    }

    pub fn file_path(&self) -> String {
        format!("{}/{}", self.model_dir_path, self.model_filename)
    }
}

#[derive(Debug, Clone)]
pub struct Scene {
    models: Vec<Model>,
}

impl Scene {
    pub fn new() -> Self {
        Self { models: Vec::new() }
    }

    pub fn models(&self) -> &[Model] {
        &self.models
    }

    pub fn models_mut(&mut self) -> &mut [Model] {
        &mut self.models
    }

    pub fn push_model(&mut self, model: Model) {
        self.models.push(model);
    }
}
