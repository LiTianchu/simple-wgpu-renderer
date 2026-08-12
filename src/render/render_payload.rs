use crate::{
    ds::{model::Model, wgpu_resource::BindGroupLayoutState},
    utils::buffer_factory,
};
use glam::Vec3;
use wgpu::util::DeviceExt;
pub struct RenderPayload {
    pub depth_attachment_texture: wgpu::Texture,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub transform_bind_group: wgpu::BindGroup,
    pub mat_light_bind_group: wgpu::BindGroup,
}

pub fn get_initial_render_payload(
    device: &wgpu::Device,
    initial_model: &Model,
    output_width: u32,
    output_height: u32,
    bind_group_layouts: &BindGroupLayoutState,
) -> RenderPayload {
    let vertices_slice = initial_model.mesh().verts();

    let depth_output_texture_descriptor = wgpu::TextureDescriptor {
        label: Some("Output Depth Texture"),
        size: wgpu::Extent3d {
            width: output_width,
            height: output_height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    };

    let depth_attachment_texture: wgpu::Texture =
        device.create_texture(&depth_output_texture_descriptor);

    let vertex_buffer_init_descriptor = wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(vertices_slice),
        usage: wgpu::BufferUsages::VERTEX,
    };

    let vertex_buffer = device.create_buffer_init(&vertex_buffer_init_descriptor);

    let face_slice: &[u8] = bytemuck::cast_slice(initial_model.mesh().faces());
    let index_buffer_init_descriptor = wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: face_slice,
        usage: wgpu::BufferUsages::INDEX,
    };

    let index_buffer = device.create_buffer_init(&index_buffer_init_descriptor);

    let mvp_uniform_buffer = buffer_factory::create_mvp_uniform_buffer(
        &device,
        Vec3 {
            x: 5.0,
            y: 5.0,
            z: 5.0,
        },
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        Vec3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
        30.0,
        1.0,
        1.0,
        1000.0,
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
        depth_attachment_texture,
        vertex_buffer,
        index_buffer,
        transform_bind_group,
        mat_light_bind_group,
    }
}
