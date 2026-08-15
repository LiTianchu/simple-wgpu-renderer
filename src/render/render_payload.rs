use crate::{
    ds::{
        model::{Model, TextureObject},
        transformation::{CameraInfo, ObjectTransform, ProjectionInfo},
        wgpu_resource::SceneBindGroupLayoutSet,
    },
    render::factory::buffer_factory,
};
use glam::Vec3;
use wgpu::util::DeviceExt;
pub struct RenderPayload {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub transform_bind_group: wgpu::BindGroup,
    pub light_bind_group: wgpu::BindGroup,
}

pub fn create_standard_render_payload(
    device: &wgpu::Device,
    // model: &Model,
    scene_bind_group_layouts: &SceneBindGroupLayoutSet,
    object_transform: &ObjectTransform,
    camera_info: &CameraInfo,
    projection_info: &ProjectionInfo,
    render_width: u32,
    render_height: u32,
) -> RenderPayload {
    // let vertices = model.all_verts_copied();
    // let vertices_slice = vertices.as_slice();

    let vertex_buffer_init_descriptor = wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: &[],
        usage: wgpu::BufferUsages::VERTEX,
    };

    let vertex_buffer = device.create_buffer_init(&vertex_buffer_init_descriptor);

    // let faces = model.all_faces_copied();
    // let face_slice: &[u8] = bytemuck::cast_slice(faces.as_slice());
    let index_buffer_init_descriptor = wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: &[],
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

    // let material_uniform_buffer = buffer_factory::create_material_uniform_buffer(
    //     &device,
    //     [149, 191, 201, 255], // muted greenish-blue color
    // );

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
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: light_uniform_buffer.as_entire_binding(),
            },
        ],
    });


    RenderPayload {
        vertex_buffer,
        index_buffer,
        transform_bind_group,
        light_bind_group,
    }
}
