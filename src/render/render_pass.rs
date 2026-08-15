use anyhow::{Context, anyhow};

use crate::ds::model::{Face, MaterialStore, Scene, TextureStore, Vertex};

pub fn render_to_output_buffer(
    // WGPU Resources
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pipeline: &wgpu::RenderPipeline,

    // Global Drawing Resources
    transform_bind_group: &wgpu::BindGroup,
    light_bind_group: &wgpu::BindGroup,
    clear_color: wgpu::Color,

    // Model Specific Drawing Resources
    vertex_buffer: &wgpu::Buffer,
    index_buffer: &wgpu::Buffer,
    material_store: &MaterialStore,
    texture_store: &mut TextureStore,
    scene: &Scene,

    // Output Description
    color_output_texture: &wgpu::Texture,
    depth_output_texture: &wgpu::Texture,
    copy_width: u32,
    copy_height: u32,
    receiver_buffer_bytes_per_row: u32,
    receiver_buffer_row_num: u32,

    // Receiver buffer
    receiver_buffer: &wgpu::Buffer,
) -> anyhow::Result<wgpu::SubmissionIndex> {
    let depth_output_texture_view =
        depth_output_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let color_output_texture_view_descriptor = wgpu::TextureViewDescriptor::default();

    let color_output_texture_view: wgpu::TextureView =
        color_output_texture.create_view(&color_output_texture_view_descriptor);

    let command_encoder_descriptor = wgpu::CommandEncoderDescriptor {
        label: Some("Render to Output Buffer Command Encoder"),
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

        println!("Rendering {} modes", scene.models().len());

        let mut index_buffer_offset: wgpu::BufferAddress = 0;
        let mut vertex_buffer_offset: wgpu::BufferAddress = 0;
        let mut index_count_offset: u32 = 0;
        let mut base_vertex: i32 = 0;

        for model in scene.models_iter() {
            for mesh in model.meshes().iter() {
                // ============ Draw Call ============
                // TODO: Handle no material situation (default material generation)
                let mat_key = mesh.mat_key().unwrap();
                let mat_obj = material_store.get_material(mat_key).unwrap();
                let mat_bind_group = &mat_obj.material_bind_group;

                // TODO: Handle no texture situation (default texture generation)
                // TODO: Handle multiple texture types (normal, specular, shininess)
                let diffuse_texture_sub_path = mat_obj
                    .material
                    .texture_set
                    .diffuse_map_path
                    .as_ref()
                    .ok_or("Diffuse map path not found")
                    .map_err(|e| anyhow!("Error occurred while loading diffuse texture: {}", e))?;
                let diffuse_texture_full_path =
                    format!("{}/{}", model.model_dir_path(), diffuse_texture_sub_path);

                let texture_label = format!(
                    "{}-{}-{}",
                    model.file_path(),
                    mat_key,
                    diffuse_texture_sub_path
                );
                let texture_obj = texture_store
                    .get_or_load_texture(
                        device,
                        queue,
                        diffuse_texture_full_path.clone(),
                        wgpu::TextureFormat::Rgba8Unorm,
                        texture_label,
                    )
                    .with_context(|| {
                        format!(
                            "Failed to load texture: {}",
                            diffuse_texture_full_path.clone()
                        )
                    })?;
                let texture_sampler_bind_group = &texture_obj.texture_sampler_bind_group;

                render_pass.set_bind_group(0, transform_bind_group, &[]);
                render_pass.set_bind_group(1, light_bind_group, &[]);
                render_pass.set_bind_group(2, mat_bind_group, &[]);

                render_pass.set_bind_group(3, texture_sampler_bind_group, &[]);

                // write buffer
                queue.write_buffer(
                    vertex_buffer,
                    vertex_buffer_offset,
                    bytemuck::cast_slice(mesh.verts()),
                );
                queue.write_buffer(
                    index_buffer,
                    index_buffer_offset,
                    bytemuck::cast_slice(mesh.faces()),
                );
                vertex_buffer_offset +=
                    (std::mem::size_of::<Vertex>() * mesh.verts().len()) as wgpu::BufferAddress;
                index_buffer_offset +=
                    (std::mem::size_of::<Face>() * mesh.faces().len()) as wgpu::BufferAddress;

                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(
                    index_count_offset..(index_count_offset + mesh.draw_indices()),
                    base_vertex,
                    0..1,
                );
                index_count_offset += mesh.draw_indices();

                base_vertex += mesh.verts().len() as i32
                // ============ End Draw Call ============
            }
        }
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
    Ok(submission_index)
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
    light_bind_group: &wgpu::BindGroup,
    clear_color: wgpu::Color,

    vertex_buffer: &wgpu::Buffer,
    index_buffer: &wgpu::Buffer,
    material_store: &MaterialStore,
    texture_store: &mut TextureStore,
    scene: &Scene,

    // Attachment
    depth_output_texture: &wgpu::Texture,

    // UI
    egui_renderer: &mut egui_wgpu::Renderer,
    paint_jobs: &[egui::ClippedPrimitive],
    textures_delta: &mut egui::TexturesDelta,
    screen_descriptor: &egui_wgpu::ScreenDescriptor,
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

    // upload egui textures
    for (id, image_deltas) in textures_delta.set.drain() {
        for image_delta in image_deltas {
            egui_renderer.update_texture(device, queue, id, &image_delta);
        }
    }

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Window Command Encoder"),
    });

    // 3D scene render pass
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
        let mut index_buffer_offset: wgpu::BufferAddress = 0;
        let mut vertex_buffer_offset: wgpu::BufferAddress = 0;
        let mut index_count_offset: u32 = 0;
        let mut base_vertex: i32 = 0;
        for model in scene.models_iter() {
            for mesh in model.meshes().iter() {
                // ============ Draw Call ============
                // TODO: Handle no material situation (default material generation)
                let mat_key = mesh.mat_key().unwrap();
                let mat_obj = material_store.get_material(mat_key).unwrap();
                let mat_bind_group = &mat_obj.material_bind_group;

                // TODO: Handle no texture situation (default texture generation)
                // TODO: Handle multiple texture types (normal, specular, shininess)
                let diffuse_texture_sub_path =
                    mat_obj.material.texture_set.diffuse_map_path.as_ref()?;
                let diffuse_texture_full_path =
                    format!("{}/{}", model.model_dir_path(), diffuse_texture_sub_path);

                let texture_label = format!(
                    "{}-{}-{}",
                    model.file_path(),
                    mat_key,
                    diffuse_texture_sub_path
                );
                let texture_obj = texture_store
                    .get_or_load_texture(
                        device,
                        queue,
                        diffuse_texture_full_path.clone(),
                        wgpu::TextureFormat::Rgba8Unorm,
                        texture_label,
                    )
                    .with_context(|| {
                        format!(
                            "Failed to load texture: {}",
                            diffuse_texture_full_path.clone()
                        )
                    })
                    .ok()?;
                let texture_sampler_bind_group = &texture_obj.texture_sampler_bind_group;

                render_pass.set_bind_group(0, transform_bind_group, &[]);
                render_pass.set_bind_group(1, light_bind_group, &[]);
                render_pass.set_bind_group(2, mat_bind_group, &[]);

                render_pass.set_bind_group(3, texture_sampler_bind_group, &[]);
                // write buffer
                queue.write_buffer(
                    vertex_buffer,
                    vertex_buffer_offset,
                    bytemuck::cast_slice(mesh.verts()),
                );
                queue.write_buffer(
                    index_buffer,
                    index_buffer_offset,
                    bytemuck::cast_slice(mesh.faces()),
                );
                vertex_buffer_offset +=
                    (std::mem::size_of::<Vertex>() * mesh.verts().len()) as wgpu::BufferAddress;
                index_buffer_offset +=
                    (std::mem::size_of::<Face>() * mesh.faces().len()) as wgpu::BufferAddress;

                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(
                    index_count_offset..(index_count_offset + mesh.draw_indices()),
                    base_vertex,
                    0..1,
                );
                index_count_offset += mesh.draw_indices();
                base_vertex += mesh.verts().len() as i32
                // ============ End Draw Call ============
            }
        }
    }

    // egui command buffer
    let user_command_buffers =
        egui_renderer.update_buffers(device, queue, &mut encoder, paint_jobs, screen_descriptor);

    // egui render pass
    {
        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("EGUI Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    // preseve the 3D scene underneath
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        egui_renderer.render(
            &mut render_pass.forget_lifetime(),
            paint_jobs,
            screen_descriptor,
        );
    }

    // submit both 3D pass and UI pass
    let submission_index = queue.submit(user_command_buffers.into_iter().chain([encoder.finish()]));

    queue.present(output);
    for id in textures_delta.free.drain() {
        egui_renderer.free_texture(&id);
    }

    Some(submission_index)
}
