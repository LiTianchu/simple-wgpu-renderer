use my_renderer::constants;
use my_renderer::ds::model::{MaterialStore, Scene, TextureStore};
use my_renderer::ds::wgpu_resource::{WgpuObject,SceneBindGroupLayoutSet};
use my_renderer::io::{file_op, image_exporter, model_loader};
use my_renderer::runner::run;
use std::env;

fn main() -> anyhow::Result<()> {
    let arg_list: Vec<String> = env::args().collect();
    let arg_len = arg_list.len();

    let mut window_mode = false;
    let mut model_path = constants::DEFAULT_MODEL_PATH.to_string();

    let loaded_scene: Scene;
    for i in 0..arg_len {
        let arg_str = arg_list[i].as_ref();
        match arg_str {
            "-w" => window_mode = true,
            "-m" => {
                if i + 1 == arg_len {
                    panic!("No model path specificed after -m!");
                }

                model_path = arg_list[i + 1].clone();
            }
            _ => {}
        };
    }

    let all_model_paths = file_op::get_files_by_type_recur(model_path.clone(), "obj")?;

    if all_model_paths.is_empty() {
        panic!("No OBJ model found at path: {}", &model_path);
    }

    let mat_bind_grp_layout_descriptor = wgpu::BindGroupLayoutDescriptor {
        label: Some("Material Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0, // material
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

    let wgpu_state = WgpuObject::on_screen(screen_info.unwrap()).await;
    let wgpu_state = WgpuObject::off_screen().await;

    let material_bind_grp_layout = wgpu_state
        .device
        .create_bind_group_layout(&mat_bind_grp_layout_descriptor);

    let mut material_store = MaterialStore::new(material_bind_grp_layout);

    loaded_scene = model_loader::load_obj_models_to_scene(all_model_paths, &mut material_store, &wgpu_state.device)
        .expect(&format!("No model loaded at path: {}", &model_path));

    println!(
        "Materials in store: {:?}",
        material_store.materials().keys()
    );


    let texture_sampler_bind_grp_layout_descriptor = wgpu::BindGroupLayoutDescriptor {
        label: Some("Texture-Sampler Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    };
    let texture_sampler_bind_grp_layout = wgpu_state
        .device
        .create_bind_group_layout(&texture_sampler_bind_grp_layout_descriptor);

    let transform_bind_grp_layout_descriptor = wgpu::BindGroupLayoutDescriptor {
        label: Some("Transform Bind Group Layout"),
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

    let light_bind_grp_layout_descriptor = wgpu::BindGroupLayoutDescriptor {
        label: Some("Light Bind Group Layout"),
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
        ],
    };

    let transform_bind_grp_layout = wgpu_state
        .device
        .create_bind_group_layout(&transform_bind_grp_layout_descriptor);

    let light_bind_grp_layout = wgpu_state
        .device
        .create_bind_group_layout(&light_bind_grp_layout_descriptor);

    let scene_bind_group_layout = SceneBindGroupLayoutSet{
        transform_bind_group_layout,
        light_bind_group_layout,
    };

    let mut texture_store = TextureStore::new(texture_sampler_bind_grp_layout);

    if window_mode {
        run(wgpu_state,loaded_scene, material_store, texture_store, scene_bind_group_layout).expect("Failed to run the application");
        Ok(())
    } else {
        // export image
        let image_export_dir = std::path::Path::new(constants::IMAGE_EXPORT_DIR);

        if !image_export_dir.exists() {
            std::fs::create_dir(image_export_dir).expect(&format!(
                "Failed to create output directory: {:?}",
                constants::IMAGE_EXPORT_DIR
            ));
        }

        println!("Num Models in scene: {}", loaded_scene.models().len(),);
        pollster::block_on(image_exporter::render_image(
            &loaded_scene,
            &material_store,
            &mut texture_store,
            &scene_bind_group_layout,
            image_export_dir,
            constants::IMAGE_FILE_NAME,
            constants::IMAGE_FILE_FORMAT,
            constants::TEXTURED_VERT_SHADER_PATH,
            constants::TEXTURED_FRAG_SHADER_PATH,
            constants::IMAGE_EXPORT_WIDTH,
            constants::IMAGE_EXPORT_HEIGHT,
        ))?;
        Ok(())
    }
}
