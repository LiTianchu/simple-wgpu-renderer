use crate::ds::model::{Face, Model, Vertex};
use crate::render::{render_pass, render_payload};
use crate::utils::{buffer_factory, copying, render_setup_factory};
use glam::Vec3;
use std::path::Path;
use wgpu::util::DeviceExt;

const BYTES_PER_PIXEL: usize = 4;

pub async fn render_image(
    model: &Model,
    export_dir: impl AsRef<Path>,
    export_file_name: impl Into<String>,
    export_file_ext: impl Into<String>,
    vert_shader_path: impl AsRef<Path>,
    frag_shader_path: impl AsRef<Path>,
    output_width: u32,
    output_height: u32,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        output_width > 0 && output_height > 0,
        "Image should not have zero size!"
    );
    let vertices = model.mesh().verts();
    let faces = model.mesh().faces();

    let renderer_state = render_setup_factory::create_render_setup_raster_standard(
        vert_shader_path,
        frag_shader_path,
        None,
    )
    .await?;

    let device = renderer_state.wgpu_object.device;

    let render_payload = render_payload::get_initial_render_payload(
        &device,
        model,
        output_width,
        output_height,
        &renderer_state.bind_group_layouts,
    );

    let unpadded_bytes_per_row: u32 = output_width * (BYTES_PER_PIXEL as u32);
    let alignment: u32 = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;

    let padded_byte_per_row: u32 = unpadded_bytes_per_row.div_ceil(alignment) * alignment;
    let output_buffer_size: u64 =
        padded_byte_per_row as wgpu::BufferAddress * output_height as wgpu::BufferAddress;

    let output_buffer_descriptor = wgpu::BufferDescriptor {
        label: Some("Image Export Output Buffer"),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
        size: output_buffer_size,
    };

    let output_buffer: wgpu::Buffer = device.create_buffer(&output_buffer_descriptor);

    let color_output_texture_descriptor = wgpu::TextureDescriptor {
        label: Some("Image Export Output Texture"),
        size: wgpu::Extent3d {
            width: output_width,
            height: output_height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: renderer_state.frag_texture_format,
        usage: wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    };

    let color_output_texture: wgpu::Texture =
        device.create_texture(&color_output_texture_descriptor);

    let depth_output_texture_descriptor = wgpu::TextureDescriptor {
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

    let depth_output_texture: wgpu::Texture =
        device.create_texture(&depth_output_texture_descriptor);

    let render_payload = render_payload::get_initial_render_payload(
        &device,
        model,
        output_width,
        output_height,
        &renderer_state.bind_group_layouts,
    );

    let submission_index: wgpu::SubmissionIndex = render_pass::render_to_output_buffer(
        &device,
        &renderer_state.wgpu_object.queue,
        &renderer_state.render_pipeline,
        &render_payload.transform_bind_group,
        &render_payload.mat_light_bind_group,
        &render_payload.vertex_buffer,
        &render_payload.index_buffer,
        0..(faces.len() * 3) as u32,
        wgpu::Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        },
        &color_output_texture,
        &depth_output_texture,
        output_width,
        output_height,
        padded_byte_per_row,
        output_height,
        &output_buffer,
    );

    let output_buffer_slice: wgpu::BufferSlice = output_buffer.slice(..);

    let output_img_pixels: Vec<u8> = copying::buffer_slice_to_byte_array(
        output_buffer_slice,
        submission_index,
        &device,
        output_height,
        padded_byte_per_row,
        unpadded_bytes_per_row,
    )?;

    output_buffer.unmap();

    let export_location: String = format!(
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
