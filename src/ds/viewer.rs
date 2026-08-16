use std::sync::Arc;
use winit::window::Window;

use crate::constants;
pub struct Screen {
    pub window: Arc<Window>,
    pub window_inner_width: u32,
    pub window_inner_height: u32,
}

pub struct ViewerState {
    pub model_rotation_euler_deg: glam::Vec3,
    pub model_scale_uniform: f32,
    pub cam_azimuth_deg: f32,
    pub cam_elevation_deg: f32,
    pub cam_radius: f32,
    pub cam_fov_deg: f32,
    pub light_direction: glam::Vec3,
    pub light_energy: f32,
    pub ambient_contribution: f32,
}

impl Default for ViewerState {
    fn default() -> Self {
        Self {
            model_rotation_euler_deg: glam::Vec3::new(0.0, 0.0, 0.0),
            model_scale_uniform: 1.0,
            cam_azimuth_deg: 90.0, // place camera at positive z
            cam_elevation_deg: 45.0,
            cam_radius: 10.0,
            cam_fov_deg: 45.0,
            light_direction: constants::INITIAL_LIGHT_DIR,
            light_energy: constants::INITIAL_LIGHT_ENERGY,
            ambient_contribution: constants::INITIAL_AMBIENT_CONTRIBUTION,
        }
    }
}

pub struct EguiFrame {
    pub paint_jobs: Vec<egui::ClippedPrimitive>,
    pub textures_delta: egui::TexturesDelta,
    pub screen_descriptor: egui_wgpu::ScreenDescriptor,
}
