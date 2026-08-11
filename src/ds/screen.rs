use std::sync::Arc;
use winit::window::Window;
pub struct Screen {
    pub window: Arc<Window>,
    pub window_inner_width: u32,
    pub window_inner_height: u32,
}
