use std::sync::Arc;
use winit::window::Window;
pub struct Screen {
    pub window: Arc<Window>,
    pub window_inner_width: u32,
    pub window_inner_height: u32,
}

pub struct ViewerState {
    pub rotation_euler: glam::Vec3,
}

pub struct EguiFrame {
    pub paint_jobs: Vec<egui::ClippedPrimitive>,
    pub textures_delta: egui::TexturesDelta,
    pub screen_descriptor: egui_wgpu::ScreenDescriptor,
}
