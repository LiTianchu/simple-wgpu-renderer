use crate::{ds::model::TextureObject, util::copying};

pub fn create_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    rgba_data: &[u8],
    dimensions: (u32, u32),
    texture_format: wgpu::TextureFormat,
    texture_label: impl Into<String>,
) -> TextureObject {
    let texture_size = wgpu::Extent3d {
        width: dimensions.0,
        height: dimensions.1,
        depth_or_array_layers: 1,
    };
    let texture_label = texture_label.into();
    let sampler_label = format!("{} Sampler", &texture_label);

    let texture_descriptor = wgpu::TextureDescriptor {
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: texture_format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST,
        label: Some(&texture_label),
        view_formats: &[],
    };
    let wgpu_texture = device.create_texture(&texture_descriptor);

    // NOTE: write_texture does not require byte alignment
    copying::write_texture_rgba(&queue, &wgpu_texture, dimensions, &rgba_data);

    let texture_view = wgpu_texture.create_view(&wgpu::TextureViewDescriptor::default());

    // NOTE: there are some model's UV is more than 1.0 meant for repeating
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some(&sampler_label),
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        address_mode_w: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        ..Default::default()
    });

    TextureObject {
        texture: wgpu_texture,
        texture_view,
        sampler,
    }
}
