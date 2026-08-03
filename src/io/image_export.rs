use std::path::Path;

pub async fn render_png(
    export_path: impl AsRef<Path>,
    vert_shader_path: impl AsRef<Path>,
    frag_shader_path: impl AsRef<Path>,
    output_width: u32,
    output_height: u32,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        output_width > 0 && output_height > 0,
        "Image should not have zero size!"
    );

    let wgpu_instance_descriptor: wgpu::InstanceDescriptor = wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        backend_options: Default::default(),
        flags: Default::default(),
        memory_budget_thresholds: Default::default(),
        display: None,
    };

    let wgpu_instance = wgpu::Instance::new(wgpu_instance_descriptor);

    let request_adator_options = wgpu::RequestAdapterOptions {
        power_preference: Default::default(),
        compatible_surface: None,
        force_fallback_adapter: false,
        apply_limit_buckets: true,
    };

    let wgpu_adapter: wgpu::Adapter = wgpu_instance
        .request_adapter(&request_adator_options)
        .await?;

    let device_required_features = wgpu::Features::empty();
    let device_exp_features = wgpu::ExperimentalFeatures::disabled();

    let device_descriptor = wgpu::DeviceDescriptor {
        label: Some("Image Export Device Descriptor."),
        required_features: device_required_features,
        experimental_features: device_exp_features,
        required_limits: wgpu::Limits::defaults(),
        memory_hints: wgpu::MemoryHints::default(),
        trace: wgpu::Trace::Off,
    };

    let (device, queue) = wgpu_adapter.request_device(&device_descriptor).await?;

    let texture_format = wgpu::TextureFormat::Rgba8UnormSrgb;

    let pipeline_layout_descriptor = wgpu::PipelineLayoutDescriptor {
        label: Some("Image Export Pipeline Layout"),
        bind_group_layouts: &[],
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
        label: Some("Image Export Vertex Shader"),
        source: wgpu::ShaderSource::Wgsl(vertex_shader_source.into()),
    });

    let pipeline_vert_state = wgpu::VertexState {
        module: &vert_shader_module,
        entry_point: Some("vs_main"),
        compilation_options: Default::default(),
        buffers: &[],
    };

    let frag_shader_path_ref = frag_shader_path.as_ref();
    let fragment_shader_source = std::fs::read_to_string(frag_shader_path_ref).expect(&format!(
        "Fragment shader path {:?} is invalid!",
        frag_shader_path_ref
    ));

    let frag_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Image Export Fragment Shader"),
        source: wgpu::ShaderSource::Wgsl(fragment_shader_source.into()),
    });

    let pipeline_frag_state = wgpu::FragmentState {
        module: &frag_shader_module,
        entry_point: Some("fs_main"),
        compilation_options: Default::default(),
        targets: &[Some(wgpu::ColorTargetState {
            format: texture_format,
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
        label: Some("Image Export Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: pipeline_vert_state,
        fragment: Some(pipeline_frag_state),
        primitive: pipeline_primitive_state,
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    };

    let pipeline = device.create_render_pipeline(&pipeline_descriptor);

    let output_texture_descriptor = wgpu::TextureDescriptor {
        label: Some("Image Export Output Texture"),
        size: wgpu::Extent3d {
            width: output_width,
            height: output_height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: texture_format,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    };

    let output_texture = device.create_texture(&output_texture_descriptor);

    let texture_view_descriptor = wgpu::TextureViewDescriptor::default();
    let output_texture_view = output_texture.create_view(&texture_view_descriptor);

    Ok(())
}
