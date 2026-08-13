use std::sync::Arc;
use winit::window::Window;
pub struct Screen {
    pub window: Arc<Window>,
    pub window_inner_width: u32,
    pub window_inner_height: u32,
}

pub struct ViewerState {
    pub model_rotation_euler_deg: glam::Vec3,
    pub model_scale_uniform: f32,
    pub cam_elevation_deg: f32,
    pub cam_radius: f32,
    pub cam_fov_deg: f32,
}

pub struct EguiFrame {
    pub paint_jobs: Vec<egui::ClippedPrimitive>,
    pub textures_delta: egui::TexturesDelta,
    pub screen_descriptor: egui_wgpu::ScreenDescriptor,
}
