pub fn render_to_output_buffer(
    // WGPU Resources
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pipeline: &wgpu::RenderPipeline,

    // Drawing Resources
    transform_bind_group: &wgpu::BindGroup,
    mat_light_bind_group: &wgpu::BindGroup,
    vertex_buffer: &wgpu::Buffer,
    index_buffer: &wgpu::Buffer,
    draw_indices: std::ops::Range<u32>,
    clear_color: wgpu::Color,

    // Output Description
    color_output_texture: &wgpu::Texture,
    depth_output_texture: &wgpu::Texture,
    copy_width: u32,
    copy_height: u32,
    receiver_buffer_bytes_per_row: u32,
    receiver_buffer_row_num: u32,

    // Receiver buffer
    receiver_buffer: &wgpu::Buffer,
) -> wgpu::SubmissionIndex {
    let depth_output_texture_view =
        depth_output_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let color_output_texture_view_descriptor = wgpu::TextureViewDescriptor::default();

    let color_output_texture_view: wgpu::TextureView =
        color_output_texture.create_view(&color_output_texture_view_descriptor);

    let command_encoder_descriptor = wgpu::CommandEncoderDescriptor {
        label: Some("Command Encoder Descriptor"),
    };

    let mut command_encoder: wgpu::CommandEncoder =
        device.create_command_encoder(&command_encoder_descriptor);

    let render_pass_color_attachments: [Option<wgpu::RenderPassColorAttachment>; 1] =
        [Some(wgpu::RenderPassColorAttachment {
            view: &color_output_texture_view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(clear_color),
                store: wgpu::StoreOp::Store,
            },
        })];

    let render_pass_descriptor = wgpu::RenderPassDescriptor {
        label: Some("Image Export Render Pass"),
        color_attachments: &render_pass_color_attachments,
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: &depth_output_texture_view,
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

    {
        let mut render_pass: wgpu::RenderPass =
            command_encoder.begin_render_pass(&render_pass_descriptor);
        render_pass.set_pipeline(&pipeline);
        render_pass.set_bind_group(0, transform_bind_group, &[]);
        render_pass.set_bind_group(1, mat_light_bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(draw_indices, 0, 0..1);
    }

    command_encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &color_output_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &receiver_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(receiver_buffer_bytes_per_row),
                rows_per_image: Some(receiver_buffer_row_num),
            },
        },
        wgpu::Extent3d {
            width: copy_width,
            height: copy_height,
            depth_or_array_layers: 1,
        },
    );

    let submission_index = queue.submit([command_encoder.finish()]);
    submission_index
}

pub fn render_to_screen(
    // WGPU resources
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    surface: &wgpu::Surface,
    surface_config: &wgpu::SurfaceConfiguration,
    render_pipeline: &wgpu::RenderPipeline,

    // Drawing resources
    transform_bind_group: &wgpu::BindGroup,
    mat_light_bind_group: &wgpu::BindGroup,
    vertex_buffer: &wgpu::Buffer,
    index_buffer: &wgpu::Buffer,
    draw_indices: std::ops::Range<u32>,
    clear_color: wgpu::Color,

    // Attachment
    depth_output_texture: &wgpu::Texture,
) -> Option<wgpu::SubmissionIndex> {
    let output = match surface.get_current_texture() {
        wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
        wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => surface_texture,
        wgpu::CurrentSurfaceTexture::Timeout
        | wgpu::CurrentSurfaceTexture::Occluded
        | wgpu::CurrentSurfaceTexture::Validation => {
            // skip this frame
            return None;
        }
        wgpu::CurrentSurfaceTexture::Outdated => {
            surface.configure(device, surface_config);
            return None;
        }
        wgpu::CurrentSurfaceTexture::Lost => {
            panic!("Device is lost during window render!")
        }
    };

    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let depth_output_texture_view =
        depth_output_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Window Command Encoder"),
    });

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Window Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_output_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Discard,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });

        render_pass.set_pipeline(render_pipeline);
        render_pass.set_bind_group(0, transform_bind_group, &[]);
        render_pass.set_bind_group(1, mat_light_bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(draw_indices, 0, 0..1);
    }

    // submit will accept anything that implements IntoIter
    let submission_index = queue.submit(std::iter::once(encoder.finish()));
    queue.present(output);
    Some(submission_index)
}
