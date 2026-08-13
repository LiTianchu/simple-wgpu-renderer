use crate::{
    ds::{
        viewer::Screen,
        wgpu_resource::{BindGroupLayoutState, RendererState, WgpuObject},
    },
    render::factory::render_pipeline_factory,
};
use std::path::Path;

pub async fn create_render_setup_raster_standard(
    vert_shader_path: impl AsRef<Path>,
    frag_shader_path: impl AsRef<Path>,
    screen_info: Option<Screen>,
) -> anyhow::Result<RendererState> {
    let screen_info_is_none = screen_info.is_none();
    let wgpu_state = if screen_info_is_none {
        WgpuObject::off_screen().await
    } else {
        WgpuObject::on_screen(screen_info.unwrap()).await
    };

    let texture_format = if screen_info_is_none {
        wgpu::TextureFormat::Rgba8UnormSrgb
    } else {
        wgpu_state.surface_state.as_ref().unwrap().config.format
    };

    let transform_bind_grp_layout_descriptor = wgpu::BindGroupLayoutDescriptor {
        label: Some("Image Export Transform Bind Group Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0, // transforms
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    };

    let mat_light_bind_grp_layout_descriptor = wgpu::BindGroupLayoutDescriptor {
        label: Some("Image Export Material-Light Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0, // light
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1, // material
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    };

    let transform_bind_grp_layout = wgpu_state
        .device
        .create_bind_group_layout(&transform_bind_grp_layout_descriptor);
    let mat_light_bind_grp_layout = wgpu_state
        .device
        .create_bind_group_layout(&mat_light_bind_grp_layout_descriptor);

    let pipeline = render_pipeline_factory::create_render_pipeline_raster(
        &wgpu_state.device,
        &texture_format,
        &[
            Some(&transform_bind_grp_layout),
            Some(&mat_light_bind_grp_layout),
        ],
        vert_shader_path.as_ref(),
        frag_shader_path.as_ref(),
    )
    .await?;

    let renderer_state = RendererState {
        wgpu_object: wgpu_state,
        render_pipeline: pipeline,
        frag_texture_format: texture_format,
        bind_group_layouts: BindGroupLayoutState {
            transform_bind_group_layout: transform_bind_grp_layout,
            mat_light_bind_group_layout: mat_light_bind_grp_layout,
        },
    };

    Ok(renderer_state)
}
