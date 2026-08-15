use winit::dpi::PhysicalSize;
pub const TEXTURED_VERT_SHADER_PATH: &str = "./src/shaders/textured.wgsl";
pub const TEXTURED_FRAG_SHADER_PATH: &str = "./src/shaders/textured.wgsl";
pub const UV_VERT_SHADER_PATH: &str = "./src/shaders/uv.wgsl";
pub const UV_FRAG_SHADER_PATH: &str = "./src/shaders/uv.wgsl";
pub const FLAT_VERT_SHADER_PATH: &str = "./src/shaders/flat_color.wgsl";
pub const FLAT_FRAG_SHADER_PATH: &str = "./src/shaders/flat_color.wgsl";
pub const WINDOW_PHYSICAL_SIZE: PhysicalSize<u32> = PhysicalSize::new(1280, 800);

pub const IMAGE_EXPORT_DIR: &str = "./output";
pub const IMAGE_FILE_NAME: &str = "render";
pub const IMAGE_FILE_FORMAT: &str = "png";
pub const DEFAULT_MODEL_PATH: &str = "./assets/obj/mini_forest/tree-high.obj";
pub const IMAGE_EXPORT_WIDTH: u32 = 1280;
pub const IMAGE_EXPORT_HEIGHT: u32 = 800;
