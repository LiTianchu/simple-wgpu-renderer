use my_renderer::constants;
use my_renderer::ds::model::{MaterialStore, Scene, TextureStore};
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

    let mut material_store = MaterialStore::new();

    loaded_scene = model_loader::load_obj_models_to_scene(all_model_paths, &mut material_store)
        .expect(&format!("No model loaded at path: {}", &model_path));

    println!(
        "Materials in store: {:?}",
        material_store.materials().keys()
    );

    let mut texture_store = TextureStore::new();

    if window_mode {
        run(loaded_scene, material_store, texture_store).expect("Failed to run the application");
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
