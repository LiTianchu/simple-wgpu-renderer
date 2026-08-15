use crate::{
    constants,
    ds::{
        model::Vertex,
        transformation::{CameraInfo, ObjectTransform, ProjectionInfo},
        wgpu_resource::SceneBindGroupLayoutSet,
    },
    render::factory::buffer_factory,
};
use glam::Vec3;
pub struct RenderPayload {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub transform_bind_group: wgpu::BindGroup,
    pub light_bind_group: wgpu::BindGroup,
}

pub fn create_standard_render_payload(
    device: &wgpu::Device,
    scene_bind_group_layouts: &SceneBindGroupLayoutSet,
    object_transform: &ObjectTransform,
    camera_info: &CameraInfo,
    projection_info: &ProjectionInfo,
    render_width: u32,
    render_height: u32,
) -> RenderPayload {
    let vertex_buffer_init_descriptor = wgpu::BufferDescriptor {
        label: Some("Vertex Buffer"),
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        size: constants::INITIAL_BUFFER_INDEX_COUNT * std::mem::size_of::<Vertex>() as u64,
        mapped_at_creation: false,
    };

    let vertex_buffer = device.create_buffer(&vertex_buffer_init_descriptor);

    let index_buffer_init_descriptor = wgpu::BufferDescriptor {
        label: Some("Index Buffer"),
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        size: constants::INITIAL_BUFFER_INDEX_COUNT * std::mem::size_of::<u32>() as u64,
        mapped_at_creation: false,
    };

    let index_buffer = device.create_buffer(&index_buffer_init_descriptor);

    let mvp_uniform_buffer = buffer_factory::create_mvp_uniform_buffer(
        &device,
        object_transform,
        camera_info,
        projection_info,
        render_width as f32 / render_height as f32,
    );

    let light_uniform_buffer = buffer_factory::create_light_uniform_buffer(
        &device,
        Vec3 {
            x: -1.23,
            y: -1.5,
            z: -1.0,
        },
    );

    let transform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Transform Bind Group"),
        layout: &scene_bind_group_layouts.transform_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: mvp_uniform_buffer.as_entire_binding(),
        }],
    });

    let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Light Bind Group"),
        layout: &scene_bind_group_layouts.light_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: light_uniform_buffer.as_entire_binding(),
        }],
    });

    RenderPayload {
        vertex_buffer,
        index_buffer,
        transform_bind_group,
        light_bind_group,
    }
}
