use std::env;
use wgpu_tutorial::io::image_export;
use wgpu_tutorial::runner::run;
#[cfg(target_arch = "wasm32")]
use wgpu_tutorial::runner::run_web;

const IMAGE_EXPORT_DIR: &str = "./output";
const IMAGE_FILE_NAME: &str = "render";
const IMAGE_FILE_FORMAT: &str = "png";
const DEMO_VERT_SHADER_PATH: &str = "./src/shaders/triangle.wgsl";
const DEMO_FRAG_SHADER_PATH: &str = "./src/shaders/triangle.wgsl";

fn main() -> anyhow::Result<()> {
    let arg_list: Vec<String> = env::args().collect();

    if arg_list.contains(&"-w".to_string()) {
        #[cfg(target_arch = "wasm32")]
        run_web().expect("Failed to run the application in web mode");

        #[cfg(not(target_arch = "wasm32"))]
        run().expect("Failed to run the application");
    }
    let image_export_dir = std::path::Path::new(IMAGE_EXPORT_DIR);

    if !image_export_dir.exists() {
        std::fs::create_dir(image_export_dir).expect(&format!(
            "Failed to create output directory: {:?}",
            IMAGE_EXPORT_DIR
        ));
    }

    pollster::block_on(image_export::render_png(
        image_export_dir,
        IMAGE_FILE_NAME,
        IMAGE_FILE_FORMAT,
        0..3,
        DEMO_VERT_SHADER_PATH,
        DEMO_FRAG_SHADER_PATH,
        500,
        500,
    ))?;
    Ok(())
}
