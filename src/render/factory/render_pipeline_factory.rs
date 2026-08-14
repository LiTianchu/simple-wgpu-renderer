use crate::ds::model::Vertex;
use std::path::Path;

pub async fn create_render_pipeline_raster(
    device: &wgpu::Device,
    texture_format: &wgpu::TextureFormat,
    bind_group_layouts: &[Option<&wgpu::BindGroupLayout>],
    vert_shader_path: impl AsRef<Path>,
    frag_shader_path: impl AsRef<Path>,
) -> anyhow::Result<wgpu::RenderPipeline> {
    let pipeline_layout_descriptor = wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: bind_group_layouts,
        immediate_size: 0,
    };

    let pipeline_layout = device.create_pipeline_layout(&pipeline_layout_descriptor);

    let vertex_shader_path_ref = vert_shader_path.as_ref();
    let vertex_shader_source: String =
        std::fs::read_to_string(vertex_shader_path_ref).expect(&format!(
            "Vertex shader path {:?} is invalid!",
            vertex_shader_path_ref
        ));

    let vert_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Vertex Shader"),
        source: wgpu::ShaderSource::Wgsl(vertex_shader_source.into()),
    });

    let pipeline_vert_state = wgpu::VertexState {
        module: &vert_shader_module,
        entry_point: Some("vs_main"),
        compilation_options: Default::default(),
        buffers: &[Some(Vertex::BUFFER_LAYOUT)],
    };

    let frag_shader_path_ref = frag_shader_path.as_ref();
    let fragment_shader_source = std::fs::read_to_string(frag_shader_path_ref).expect(&format!(
        "Fragment shader path {:?} is invalid!",
        frag_shader_path_ref
    ));

    let frag_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Fragment Shader"),
        source: wgpu::ShaderSource::Wgsl(fragment_shader_source.into()),
    });

    let pipeline_frag_state = wgpu::FragmentState {
        module: &frag_shader_module,
        entry_point: Some("fs_main"),
        compilation_options: Default::default(),
        targets: &[Some(wgpu::ColorTargetState {
            format: *texture_format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        })],
    };

    let pipeline_primitive_state = wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: Some(wgpu::Face::Back),
        polygon_mode: wgpu::PolygonMode::Fill,
        unclipped_depth: false,
        conservative: false,
    };

    let pipeline_descriptor = wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: pipeline_vert_state,
        fragment: Some(pipeline_frag_state),
        primitive: pipeline_primitive_state,
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::Less),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    };

    let pipeline = device.create_render_pipeline(&pipeline_descriptor);
    Ok(pipeline)
}
