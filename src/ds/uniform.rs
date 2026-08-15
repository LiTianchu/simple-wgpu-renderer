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
    // k_ambient is ignored
    pub k_diffuse: u32, // RGBA color packed into a u32 bytes
    pub k_specular: u32, // RGBA color packed into a u32 bytes
    pub k_emissive: u32, // RGBA color packed into a u32 bytes
    pub index_of_refraction: f32,
    pub shininess: f32,
    pub dissolve: f32,
    pub illumination_model: u32,
}
