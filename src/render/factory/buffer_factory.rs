use crate::{
    ds::{
        model::Material,
        transformation::{CameraInfo, ObjectTransform, ProjectionInfo},
        uniform::{LightUniform, MaterialUniform},
    },
    render::factory::uniform_factory,
};
use glam::Vec3;
use wgpu::util::DeviceExt;

pub fn create_mvp_uniform_buffer(
    device: &wgpu::Device,
    object_transform: &ObjectTransform,
    camera_info: &CameraInfo,
    projection_info: &ProjectionInfo,
    aspect_ratio: f32,
) -> wgpu::Buffer {
    let transform_uniform = uniform_factory::create_mvp_uniform(
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

pub fn create_material_uniform_buffer(device: &wgpu::Device, material: &Material) -> wgpu::Buffer {
    // k_ambient is ignored by material uniform buffer
    let material_uniform = MaterialUniform {
        k_diffuse: pack_color(material.mat_attr.k_diffuse),
        k_specular: pack_color(material.mat_attr.k_specular),
        k_emissive: pack_color(material.mat_attr.k_emissive),
        index_of_refraction: material.mat_attr.index_of_refraction,
        shininess: material.mat_attr.shininess,
        dissolve: material.mat_attr.dissolve,
        illumination_model: material.mat_attr.illumination_model,
    };

    let descriptor = wgpu::util::BufferInitDescriptor {
        label: Some("Material Uniform Buffer"),
        contents: bytemuck::bytes_of(&material_uniform),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    };
    device.create_buffer_init(&descriptor)
}

fn pack_color(c: [f32; 3]) -> u32 {
    let [r, g, b] = c.map(|v| (v * 255.0) as u8);
    u32::from_be_bytes([r, g, b, 255]) // 255 = alpha/pad byte
}
