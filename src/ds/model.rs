use std::mem;
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pos_x: f32,
    pos_y: f32,
    pos_z: f32,
}

impl Vertex {
    pub fn new(pos_x: f32, pos_y: f32, pos_z: f32) -> Self {
        Self {
            pos_x,
            pos_y,
            pos_z,
        }
    }

    pub const BUFFER_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![
            0=>Float32x3
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

#[derive(Debug, Clone)]
pub struct Mesh {
    positions: Vec<Vertex>,
    faces: Vec<Face>,
}

impl Mesh {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            faces: Vec::new(),
        }
    }
    pub fn positions(&self) -> &[Vertex] {
        &self.positions
    }

    pub fn faces(&self) -> &[Face] {
        &self.faces
    }

    pub fn push_vert(&mut self, x: f32, y: f32, z: f32) {
        self.positions.push(Vertex {
            pos_x: x,
            pos_y: y,
            pos_z: z,
        });
    }

    pub fn push_face(&mut self, index_1: u32, index_2: u32, index_3: u32) {
        self.faces.push(Face {
            index_1,
            index_2,
            index_3,
        })
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TransformUniform {
    pub model: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
    pub proj: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub light_direction: [f32; 3],
    pub _padding: f32, // matches WGSL's implicit vec3 -> 16-byte struct padding
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialUniform {
    pub base_color: u32, // RGBA color packed into a u32 bytes
}

#[derive(Debug, Clone)]
pub struct Model {
    mesh: Mesh,
}

impl Model {
    pub fn new() -> Self {
        Self { mesh: Mesh::new() }
    }

    pub fn mesh(&self) -> &Mesh {
        &self.mesh
    }

    pub fn mesh_mut(&mut self) -> &mut Mesh {
        &mut self.mesh
    }
}
