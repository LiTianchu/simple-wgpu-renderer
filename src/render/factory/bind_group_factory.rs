use crate::{ds::model::Material, render::factory::buffer_factory};

pub fn create_material_bind_group(
    device: &wgpu::Device,
    material: &Material,
    material_bind_group_layout: &wgpu::BindGroupLayout,
    label: impl Into<String>,
) -> wgpu::BindGroup {
    let material_uniform_buffer = buffer_factory::create_material_uniform_buffer(device, material);

    let mat_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(&label.into()),
        layout: material_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: material_uniform_buffer.as_entire_binding(),
        }],
    });
    return mat_bind_group;
}

pub fn create_texture_sampler_bind_group(
    device: &wgpu::Device,
    texture_view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
    texture_sampler_bind_group_layout: &wgpu::BindGroupLayout,
    label: impl Into<String>,
) -> wgpu::BindGroup {
    let texture_sampler_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(&label.into()),
        layout: texture_sampler_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });

    return texture_sampler_bind_group;
}
