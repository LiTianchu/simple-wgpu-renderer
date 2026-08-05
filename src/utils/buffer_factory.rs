use crate::ds::model::{LightUniform, MaterialUniform};
use crate::utils::transform;
use glam::Vec3;
use wgpu::util::DeviceExt;

pub fn create_mvp_uniform_buffer(
    device: &wgpu::Device,
    cam_pos: Vec3,
    cam_look_at_pos: Vec3,
    cam_up: Vec3,
    vertical_fov: f32,
    aspect_ratio: f32,
    near: f32,
    far: f32,
) -> wgpu::Buffer {
    let transform_uniform = transform::create_mvp_uniform_identity(
        cam_pos,
        cam_look_at_pos,
        cam_up,
        vertical_fov,
        aspect_ratio,
        near,
        far,
    );

    let descriptor = wgpu::util::BufferInitDescriptor {
        label: Some("MVP Uniform Buffer"),
        contents: bytemuck::bytes_of(&transform_uniform),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    };

    device.create_buffer_init(&descriptor)
}

pub fn create_light_uniform_buffer(device: &wgpu::Device, light_direction: Vec3) -> wgpu::Buffer {
    let light_uniform = LightUniform {
        light_direction: light_direction.to_array(),
        _padding: 0.0,
    };

    let descriptor = wgpu::util::BufferInitDescriptor {
        label: Some("Light Uniform Buffer"),
        contents: bytemuck::bytes_of(&light_uniform),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    };

    device.create_buffer_init(&descriptor)
}

pub fn create_material_uniform_buffer(device: &wgpu::Device, base_color: [f32; 4]) -> wgpu::Buffer {
    let material_uniform = MaterialUniform { base_color };
    let descriptor = wgpu::util::BufferInitDescriptor {
        label: Some("Material Uniform Buffer"),
        contents: bytemuck::bytes_of(&material_uniform),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    };
    device.create_buffer_init(&descriptor)
}
