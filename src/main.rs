use my_renderer::ds::model::Model;
use my_renderer::io::{image_exporter, model_loader};
use my_renderer::runner::run;
use std::env;

const IMAGE_EXPORT_DIR: &str = "./output";
const IMAGE_FILE_NAME: &str = "render";
const IMAGE_FILE_FORMAT: &str = "png";
const DEMO_VERT_SHADER_PATH: &str = "./src/shaders/flat_color.wgsl";
const DEMO_FRAG_SHADER_PATH: &str = "./src/shaders/flat_color.wgsl";
const DEFAULT_MODEL_PATH: &str = "./assets/obj/cube/cube.obj";
const IMAGE_EXPORT_WIDTH: u32 = 1280;
const IMAGE_EXPORT_HEIGHT: u32 = 800;

fn main() -> anyhow::Result<()> {
    let arg_list: Vec<String> = env::args().collect();
    let arg_len = arg_list.len();

    let mut window_mode = false;
    let mut model_path = DEFAULT_MODEL_PATH.to_string();

    let loaded_model: Model;
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

    let all_model_paths = model_loader::get_files_by_type_recur(model_path.clone(), "obj")?;

    loaded_model = model_loader::load_obj_model_all(all_model_paths)
        .expect(&format!("No model loaded at path: {}", &model_path));

    if window_mode {
        run(loaded_model).expect("Failed to run the application");
        Ok(())
    } else {
        // export image
        let image_export_dir = std::path::Path::new(IMAGE_EXPORT_DIR);

        if !image_export_dir.exists() {
            std::fs::create_dir(image_export_dir).expect(&format!(
                "Failed to create output directory: {:?}",
                IMAGE_EXPORT_DIR
            ));
        }

        println!(
            "Num Meshes: {}, Num Materials: {}, Num Vertices: {}, Num Faces: {}",
            loaded_model.meshes().len(),
            loaded_model.materials().len(),
            loaded_model.vert_count(),
            loaded_model.face_count()
        );
        pollster::block_on(image_exporter::render_image(
            &loaded_model,
            image_export_dir,
            IMAGE_FILE_NAME,
            IMAGE_FILE_FORMAT,
            DEMO_VERT_SHADER_PATH,
            DEMO_FRAG_SHADER_PATH,
            IMAGE_EXPORT_WIDTH,
            IMAGE_EXPORT_HEIGHT,
        ))?;
        Ok(())
    }
}
