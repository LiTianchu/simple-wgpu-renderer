use crate::ds::model::{Face, Vertex};
use crate::utils::buffer_factory;
use crate::utils::render_pipeline_factory;
use glam::Vec3;
use std::path::Path;
use std::sync::mpsc;
use wgpu::util::DeviceExt;

const BYTES_PER_PIXEL: usize = 4;

pub async fn render_image(
    export_dir: impl AsRef<Path>,
    export_file_name: impl Into<String>,
    export_file_ext: impl Into<String>,
    vertices: &[Vertex],
    faces: &[Face],
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
        // None for off-screen rendering, need to pass in &surface if render on screen
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

    let transform_bind_grp_layout =
        device.create_bind_group_layout(&transform_bind_grp_layout_descriptor);
    let mat_light_bind_grp_layout =
        device.create_bind_group_layout(&mat_light_bind_grp_layout_descriptor);

    let pipeline = render_pipeline_factory::create_render_pipeline_raster(
        &wgpu_instance,
        &wgpu_adapter,
        &device,
        &texture_format,
        &[
            Some(&transform_bind_grp_layout),
            Some(&mat_light_bind_grp_layout),
        ],
        vert_shader_path,
        frag_shader_path,
    )
    .await
    .expect("Failed to create rendering pipeline for image export.");

    let vertex_buffer_init_descriptor = wgpu::util::BufferInitDescriptor {
        label: Some("Image Export Vertex Buffer"),
        contents: bytemuck::cast_slice(vertices),
        usage: wgpu::BufferUsages::VERTEX,
    };

    let vertex_buffer = device.create_buffer_init(&vertex_buffer_init_descriptor);

    let face_slice: &[u8] = bytemuck::cast_slice(faces);
    let index_buffer_init_descriptor = wgpu::util::BufferInitDescriptor {
        label: Some("Image Export Index Buffer"),
        contents: face_slice,
        usage: wgpu::BufferUsages::INDEX,
    };

    let index_buffer = device.create_buffer_init(&index_buffer_init_descriptor);

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
        usage: wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    };

    let output_texture: wgpu::Texture = device.create_texture(&output_texture_descriptor);

    let texture_view_descriptor = wgpu::TextureViewDescriptor::default();
    let output_texture_view: wgpu::TextureView =
        output_texture.create_view(&texture_view_descriptor);

    let depth_texture_descriptor = wgpu::TextureDescriptor {
        label: Some("Image Export Depth Texture"),
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

    let depth_texture = device.create_texture(&depth_texture_descriptor);

    let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

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
        [149, 191, 201, 255], // red color
    );

    let unpadded_bytes_per_row: u32 = output_width * (BYTES_PER_PIXEL as u32);
    let alignment: u32 = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;

    let padded_byte_per_row: u32 = unpadded_bytes_per_row.div_ceil(alignment) * alignment;
    let output_buffer_size: u64 =
        padded_byte_per_row as wgpu::BufferAddress * output_height as wgpu::BufferAddress;

    let buffer_descriptor = wgpu::BufferDescriptor {
        label: Some("Image Export Output Buffer"),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
        size: output_buffer_size,
    };

    let output_buffer: wgpu::Buffer = device.create_buffer(&buffer_descriptor);

    let command_encoder_descriptor = wgpu::CommandEncoderDescriptor {
        label: Some("Command Encoder Descriptor"),
    };

    let mut command_encoder: wgpu::CommandEncoder =
        device.create_command_encoder(&command_encoder_descriptor);

    let render_pass_color_attachments: [Option<wgpu::RenderPassColorAttachment>; 1] =
        [Some(wgpu::RenderPassColorAttachment {
            view: &output_texture_view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.1,
                    g: 0.2,
                    b: 0.3,
                    a: 1.0,
                }),
                store: wgpu::StoreOp::Store,
            },
        })];

    {
        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: Some("Image Export Render Pass"),
            color_attachments: &render_pass_color_attachments,
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Discard,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        };

        let transform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Image Export Transform Bind Group"),
            layout: &transform_bind_grp_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: mvp_uniform_buffer.as_entire_binding(),
            }],
        });

        let mat_light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Image Export Material-Light Bind Group"),
            layout: &mat_light_bind_grp_layout,
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

        let mut render_pass: wgpu::RenderPass =
            command_encoder.begin_render_pass(&render_pass_descriptor);
        render_pass.set_pipeline(&pipeline);
        render_pass.set_bind_group(0, &transform_bind_group, &[]);
        render_pass.set_bind_group(1, &mat_light_bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..(faces.len() * 3) as u32, 0, 0..1);
    }

    command_encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &output_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &output_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_byte_per_row),
                rows_per_image: Some(output_height),
            },
        },
        wgpu::Extent3d {
            width: output_width,
            height: output_height,
            depth_or_array_layers: 1,
        },
    );

    let submission_index = queue.submit([command_encoder.finish()]);

    let buffer_slice = output_buffer.slice(..);
    // use sender receiver pattern to receive the result
    let (sender, receiver) = mpsc::channel();

    buffer_slice.map_async(
        wgpu::MapMode::Read,
        move |result: Result<(), wgpu::BufferAsyncError>| {
            let _ = sender.send(result);
        },
    );

    // waits for GPU work and invokes pending mapping callbacks
    // need to poll as CPU need to wait for the async mapping result
    device.poll(wgpu::PollType::Wait {
        submission_index: Some(submission_index),
        timeout: None,
    })?;

    //receive the result and throw error if needed
    let _ = receiver
        .recv()
        .map_err(|_| anyhow::anyhow!("Buffer mapping callback was dropped"))?;

    let mapped_data: wgpu::BufferView = buffer_slice.get_mapped_range()?;

    // use unpadded bytes for output to trim the gpu padded bytes
    let mut output_img_pixels: Vec<u8> =
        vec![0_u8; (unpadded_bytes_per_row * output_height) as usize];

    // copy row-by-row to the output img pixel array
    for (source_row, dest_row) in mapped_data
        .chunks_exact(padded_byte_per_row as usize)
        .zip(output_img_pixels.chunks_exact_mut(unpadded_bytes_per_row as usize))
    {
        dest_row.copy_from_slice(&source_row[0..unpadded_bytes_per_row as usize]);
    }

    // clean up
    drop(mapped_data);
    output_buffer.unmap();

    let export_location = format!(
        "{}/{}.{}",
        export_dir.as_ref().to_string_lossy(),
        export_file_name.into(),
        export_file_ext.into()
    );

    println!("Exported render result: {}", export_location);

    image::save_buffer(
        export_location,
        &output_img_pixels,
        output_width,
        output_height,
        image::ColorType::Rgba8,
    )?;

    Ok(())
}
