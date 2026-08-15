use my_renderer::constants;
use my_renderer::ds::model::{MaterialStore, TextureStore};
use my_renderer::ds::wgpu_resource::{SceneBindGroupLayoutSet, WgpuObject};
use my_renderer::io::{file_op, image_exporter, model_loader};
use my_renderer::runner::run;
use std::env;
use std::path::{Path, PathBuf};

fn main() -> anyhow::Result<()> {
    let arg_list: Vec<String> = env::args().collect();
    let mut window_mode = false;
    let mut model_path = constants::DEFAULT_MODEL_PATH.to_string();

    for (index, arg) in arg_list.iter().enumerate() {
        match arg.as_str() {
            "-w" => window_mode = true,
            "-m" => {
                model_path = arg_list
                    .get(index + 1)
                    .cloned()
                    .ok_or_else(|| anyhow::anyhow!("No model path specified after -m"))?;
            }
            _ => {}
        }
    }

    let model_paths = file_op::get_files_by_type_recur(&model_path, "obj")?;
    anyhow::ensure!(
        !model_paths.is_empty(),
        "No OBJ model found at path: {model_path}"
    );

    if window_mode {
        run(model_paths)
    } else {
        pollster::block_on(render_off_screen(model_paths, &model_path))
    }
}

async fn render_off_screen(model_paths: Vec<PathBuf>, model_path: &str) -> anyhow::Result<()> {
    let wgpu_object = WgpuObject::off_screen().await;
    let scene_bind_group_layouts = SceneBindGroupLayoutSet::new(&wgpu_object.device);
    let mut material_store = MaterialStore::new(&wgpu_object.device);
    let mut texture_store = TextureStore::new(&wgpu_object.device, &wgpu_object.queue);

    let scene = model_loader::load_obj_models_to_scene(
        model_paths,
        &mut material_store,
        &wgpu_object.device,
    )
    .ok_or_else(|| anyhow::anyhow!("No model loaded at path: {model_path}"))?;

    println!(
        "Materials in store: {:?}",
        material_store.materials().keys()
    );

    let image_export_dir = Path::new(constants::IMAGE_EXPORT_DIR);
    std::fs::create_dir_all(image_export_dir)?;

    println!("Num Models in scene: {}", scene.models().len());
    image_exporter::render_image(
        &scene,
        &wgpu_object,
        &material_store,
        &mut texture_store,
        &scene_bind_group_layouts,
        image_export_dir,
        constants::IMAGE_FILE_NAME,
        constants::IMAGE_FILE_FORMAT,
        constants::TEXTURED_VERT_SHADER_PATH,
        constants::TEXTURED_FRAG_SHADER_PATH,
        constants::IMAGE_EXPORT_WIDTH,
        constants::IMAGE_EXPORT_HEIGHT,
    )
    .await
}
