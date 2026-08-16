use crate::{
    constants,
    ds::{
        model::Vertex,
        transformation::{CameraInfo, ObjectTransform, ProjectionInfo},
        wgpu_resource::SceneBindGroupLayoutSet,
    },
    render::factory::{buffer_factory, uniform_factory},
};
use glam::Vec3;

#[derive(Debug, Clone)]
pub struct DrawBufferSet {
    pub vertex_buffer: wgpu::Buffer,
    pub vertex_buffer_size: u64,
    pub index_buffer: wgpu::Buffer,
    pub index_buffer_size: u64,
}

impl DrawBufferSet {
    pub fn new(device: &wgpu::Device) -> Self {
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

        Self {
            vertex_buffer,
            vertex_buffer_size: vertex_buffer_init_descriptor.size,
            index_buffer,
            index_buffer_size: index_buffer_init_descriptor.size,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SceneBindGroupSet {
    pub transform_bind_group: wgpu::BindGroup,
    pub transfrom_uniform_buffer: wgpu::Buffer,
    pub light_bind_group: wgpu::BindGroup,
    pub light_uniform_buffer: wgpu::Buffer,
}

impl SceneBindGroupSet {
    pub fn new(
        device: &wgpu::Device,
        initial_object_transform: &ObjectTransform,
        initial_camera_info: &CameraInfo,
        initial_projection_info: &ProjectionInfo,
        render_width: u32,
        render_height: u32,
        initial_light_direction: Vec3,
        initial_light_energy: f32,
        initial_ambient_contribution: f32,
        scene_bind_group_layouts: &SceneBindGroupLayoutSet,
    ) -> Self {
        let mvp_uniform_buffer = buffer_factory::create_mvp_uniform_buffer(
            &device,
            initial_object_transform,
            initial_camera_info,
            initial_projection_info,
            render_width as f32 / render_height as f32,
        );

        let light_uniform_buffer = buffer_factory::create_light_uniform_buffer(
            &device,
            initial_light_direction,
            initial_light_energy,
            initial_ambient_contribution,
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

        Self {
            transform_bind_group,
            transfrom_uniform_buffer: mvp_uniform_buffer,
            light_bind_group,
            light_uniform_buffer,
        }
    }

    pub fn set_transform(
        &mut self,
        queue: &wgpu::Queue,
        new_object_transform: &ObjectTransform,
        new_camera_info: &CameraInfo,
        new_projection_info: &ProjectionInfo,
        new_aspect_ratio: f32,
    ) {
        let new_transform_uniform = uniform_factory::create_mvp_uniform(
            new_object_transform.translation,
            new_object_transform.rotation,
            new_object_transform.scale,
            new_camera_info.position,
            new_camera_info.look_at,
            new_camera_info.up,
            new_camera_info.fov,
            new_aspect_ratio,
            new_projection_info.near,
            new_projection_info.far,
        );
        queue.write_buffer(
            &self.transfrom_uniform_buffer,
            0,
            bytemuck::bytes_of(&new_transform_uniform),
        );
    }

    pub fn set_light(
        &mut self,
        queue: &wgpu::Queue,
        new_light_direction: Vec3,
        new_light_energy: f32,
        new_ambient_contribution: f32,
    ) {
        let new_light_uniform = uniform_factory::create_light_uniform(
            new_light_direction,
            new_light_energy,
            new_ambient_contribution,
        );
        queue.write_buffer(
            &self.light_uniform_buffer,
            0,
            bytemuck::bytes_of(&new_light_uniform),
        );
    }
}
