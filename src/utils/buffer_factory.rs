use crate::ds::{
    model::{LightUniform, MaterialUniform},
    transformation::{CameraInfo, ObjectTransform, ProjectionInfo},
};
use crate::utils::transform;
use glam::Vec3;
use wgpu::util::DeviceExt;

pub fn create_mvp_uniform_buffer(
    device: &wgpu::Device,
    object_transform: &ObjectTransform,
    camera_info: &CameraInfo,
    projection_info: &ProjectionInfo,
    aspect_ratio: f32,
) -> wgpu::Buffer {
    let transform_uniform = transform::create_mvp_uniform_identity(
        object_transform.translation,
        object_transform.rotation,
        object_transform.scale,
        camera_info.position,
        camera_info.look_at,
        camera_info.up,
        camera_info.fov,
        aspect_ratio,
        projection_info.near,
        projection_info.far,
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
        light_direction: light_direction.normalize().to_array(),
        _padding: 0.0,
    };

    let descriptor = wgpu::util::BufferInitDescriptor {
        label: Some("Light Uniform Buffer"),
        contents: bytemuck::bytes_of(&light_uniform),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    };

    device.create_buffer_init(&descriptor)
}

pub fn create_material_uniform_buffer(
    device: &wgpu::Device,
    base_color_rgba: [u8; 4],
) -> wgpu::Buffer {
    let material_uniform = MaterialUniform {
        base_color: u32::from_be_bytes(base_color_rgba),
    };

    let descriptor = wgpu::util::BufferInitDescriptor {
        label: Some("Material Uniform Buffer"),
        contents: bytemuck::bytes_of(&material_uniform),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    };
    device.create_buffer_init(&descriptor)
}
