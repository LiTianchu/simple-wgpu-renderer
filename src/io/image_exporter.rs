use crate::ds::{
    model::{Face, MaterialStore, Scene, TextureObject, TextureStore},
    transformation::{CameraInfo, ObjectTransform, ProjectionInfo},
};
use crate::render::{factory::render_setup_factory, render_pass, render_payload};
use crate::utils::copying;
use std::path::Path;

const BYTES_PER_PIXEL: usize = 4;

pub async fn render_image(
    scene: &Scene,
    material_store: &MaterialStore,
    texture_store: &mut TextureStore,
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

    let renderer_state = render_setup_factory::create_render_setup_raster_standard(
        vert_shader_path,
        frag_shader_path,
        None,
        (output_width, output_height),
    )
    .await?;

    let device = renderer_state.wgpu_object.device;
    let queue = renderer_state.wgpu_object.queue;

    // copy_buffer_to_texture requires byte alignment
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

    let depth_attachment_texture_descriptor = wgpu::TextureDescriptor {
        label: Some("Output Depth Texture"),
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

    let depth_attachment_texture: wgpu::Texture =
        device.create_texture(&depth_attachment_texture_descriptor);

    let object_transform = ObjectTransform::default();

    let camera_info = CameraInfo {
        position: glam::Vec3::new(5.0, 5.0, 5.0),
        look_at: glam::Vec3::new(0.0, 0.0, 0.0),
        up: glam::Vec3::new(0.0, 1.0, 0.0),
        fov: 45.0_f32.to_radians(),
    };

    let projection_info = ProjectionInfo {
        near: 1.0,
        far: 1000.0,
    };

    // TODO: Support rendering multiple models in the scene
    let model = scene
        .models()
        .first()
        .ok_or_else(|| anyhow::anyhow!("Scene has no models to render!"))?;
    println!(
        "Rendering model: {}\n  Vert count: {}\n  Face count: {}",
        model.file_path(),
        model.vert_count(),
        model.face_count()
    );

    let temp_mesh = model
        .meshes()
        .first()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Model has no meshes to render! Model file path: {}",
                model.file_path()
            )
        })
        .unwrap();

    let material = temp_mesh
        .mat_key()
        .and_then(|mat_key| material_store.get_material(mat_key));

    let mut texture_obj: Option<&TextureObject> = None;

    // TODO: Support rendering multiple materials in the model
    if let Some(mat) = material {
        if let Some(p) = mat.texture_set.diffuse_map_path.as_ref() {
            let full_path = format!("{}/{}", model.model_dir_path(), p);

            let tex_option = texture_store.get_or_load_texture(
                &device,
                &queue,
                full_path.clone(),
                wgpu::TextureFormat::Rgba8UnormSrgb,
                "Test Texture",
            );

            match tex_option {
                Some(tex) => {
                    println!("Found diffuse texture at: {}", full_path);
                    texture_obj = Some(tex);
                }
                None => {
                    println!("No diffuse texture found at: {}", full_path);
                }
            }
        }
    }

    if texture_obj.is_none() {
        println!("No diffuse texture found for the model. Rendering without texture.");
    }

    let render_payload = render_payload::create_standard_render_payload(
        &device,
        &model,
        &renderer_state.bind_group_layouts,
        &object_transform,
        &camera_info,
        &projection_info,
        texture_obj,
        output_width,
        output_height,
    );

    let faces: Vec<&Face> = model.all_faces_iter().collect();

    let submission_index: wgpu::SubmissionIndex = render_pass::render_to_output_buffer(
        &device,
        &queue,
        &renderer_state.render_pipeline,
        &render_payload.transform_bind_group,
        &render_payload.light_bind_group,
        &render_payload.mat_bind_group,
        render_payload.texture_sampler_bind_group.as_ref(),
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
        &depth_attachment_texture,
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
