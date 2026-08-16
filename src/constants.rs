use winit::dpi::PhysicalSize;
pub const TEXTURED_VERT_SHADER_PATH: &str = "./src/shaders/standard.wgsl";
pub const TEXTURED_FRAG_SHADER_PATH: &str = "./src/shaders/standard.wgsl";
pub const UV_VERT_SHADER_PATH: &str = "./src/shaders/uv.wgsl";
pub const UV_FRAG_SHADER_PATH: &str = "./src/shaders/uv.wgsl";
pub const FLAT_VERT_SHADER_PATH: &str = "./src/shaders/flat_color.wgsl";
pub const FLAT_FRAG_SHADER_PATH: &str = "./src/shaders/flat_color.wgsl";
pub const WINDOW_PHYSICAL_SIZE: PhysicalSize<u32> = PhysicalSize::new(1280, 800);
pub const INITIAL_LIGHT_DIR: glam::Vec3 = glam::Vec3 {
    x: -1.23,
    y: -1.5,
    z: -1.0,
};
pub const INITIAL_LIGHT_ENERGY: f32 = 1.0;
pub const INITIAL_AMBIENT_CONTRIBUTION: f32 = 0.2;

// Linear corrected value of RGB(39, 44, 54) for SRGB color space
pub const WINDOW_MODE_CLEAR_COLOR: wgpu::Color = wgpu::Color {
    r: 0.02029,
    g: 0.02518,
    b: 0.03688,
    a: 1.0,
};

pub const IMAGE_EXPORT_CLEAR_COLOR: wgpu::Color = wgpu::Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 0.0,
};

pub const IMAGE_EXPORT_DIR: &str = "./output";
pub const IMAGE_FILE_NAME: &str = "render";
pub const IMAGE_FILE_FORMAT: &str = "png";
pub const DEFAULT_MODEL_PATH: &str = "./assets/obj/backpack/";
pub const IMAGE_EXPORT_WIDTH: u32 = 1280;
pub const IMAGE_EXPORT_HEIGHT: u32 = 800;

pub const INITIAL_BUFFER_VERTEX_COUNT: u64 = 1_000_000;
pub const INITIAL_BUFFER_INDEX_COUNT: u64 = 3_000_000;
pub const EGUI_PANEL_BG_COLOR: egui::Color32 = egui::Color32::from_rgb(30, 30, 30);
pub const EGUI_PANEL_OPACITY: f32 = 0.75;
pub const EGUI_PANEL_PADDING_SAME: i8 = 10;
pub const EGUI_PANEL_MARGIN_SAME: i8 = 10;
