use crate::{
    ds::wgpu_resource::{RendererState, SceneBindGroupLayoutSet, WgpuObject},
    render::factory::render_pipeline_factory,
};
use std::path::Path;

pub async fn create_render_setup_raster_standard(
    wgpu_state: &WgpuObject,
    vert_shader_path: impl AsRef<Path>,
    frag_shader_path: impl AsRef<Path>,
    depth_attachment_size: (u32, u32),
    scene_bind_group_layout: &SceneBindGroupLayoutSet,
    mat_bind_group_layout: &wgpu::BindGroupLayout,
    texture_sampler_bind_group_layout: &wgpu::BindGroupLayout,
) -> anyhow::Result<RendererState> {
    let texture_format = wgpu_state
        .surface_state
        .as_ref()
        .map(|surface_state| surface_state.config.format)
        .unwrap_or(wgpu::TextureFormat::Rgba8UnormSrgb);

    let depth_attachment_texture_descriptor = wgpu::TextureDescriptor {
        label: Some("Output Depth Texture"),
        size: wgpu::Extent3d {
            width: depth_attachment_size.0,
            height: depth_attachment_size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    };

    let depth_attachment_texture: wgpu::Texture = wgpu_state
        .device
        .create_texture(&depth_attachment_texture_descriptor);

    // TODO: Make render pipeline owns texture bind group layout configuration
    let pipeline = render_pipeline_factory::create_render_pipeline_raster(
        &wgpu_state.device,
        &texture_format,
        // REMEMBER to add the new bind group layouts to the pipeline creation function
        &[
            Some(&scene_bind_group_layout.transform_bind_group_layout),
            Some(&scene_bind_group_layout.light_bind_group_layout),
            Some(mat_bind_group_layout),
            Some(texture_sampler_bind_group_layout),
        ],
        vert_shader_path.as_ref(),
        frag_shader_path.as_ref(),
    )
    .await?;

    let renderer_state = RendererState {
        render_pipeline: pipeline,
        frag_texture_format: texture_format,
        depth_attachment_texture,
    };

    Ok(renderer_state)
}
