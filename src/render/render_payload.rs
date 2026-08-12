use crate::{
    ds::{
        model::Model,
        transformation::{CameraInfo, ObjectTransform, ProjectionInfo},
        wgpu_resource::BindGroupLayoutState,
    },
    utils::buffer_factory,
};
use glam::Vec3;
use wgpu::util::DeviceExt;
pub struct RenderPayload {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub transform_bind_group: wgpu::BindGroup,
    pub mat_light_bind_group: wgpu::BindGroup,
}

pub fn create_initial_render_payload(
    device: &wgpu::Device,
    model: &Model,
    bind_group_layouts: &BindGroupLayoutState,
    object_transform: &ObjectTransform,
    camera_info: &CameraInfo,
    projection_info: &ProjectionInfo,
    render_width: u32,
    render_height: u32,
) -> RenderPayload {
    let vertices_slice = model.mesh().verts();

    let vertex_buffer_init_descriptor = wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(vertices_slice),
        usage: wgpu::BufferUsages::VERTEX,
    };

    let vertex_buffer = device.create_buffer_init(&vertex_buffer_init_descriptor);

    let face_slice: &[u8] = bytemuck::cast_slice(model.mesh().faces());
    let index_buffer_init_descriptor = wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: face_slice,
        usage: wgpu::BufferUsages::INDEX,
    };

    let index_buffer = device.create_buffer_init(&index_buffer_init_descriptor);

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

    let material_uniform_buffer = buffer_factory::create_material_uniform_buffer(
        &device,
        [149, 191, 201, 255], // muted greenish-blue color
    );

    let transform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Image Export Transform Bind Group"),
        layout: &bind_group_layouts.transform_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: mvp_uniform_buffer.as_entire_binding(),
        }],
    });

    let mat_light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Image Export Material-Light Bind Group"),
        layout: &bind_group_layouts.mat_light_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: light_uniform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: material_uniform_buffer.as_entire_binding(),
            },
        ],
    });

    RenderPayload {
        vertex_buffer,
        index_buffer,
        transform_bind_group,
        mat_light_bind_group,
    }
}
