use crate::ds::screen::Screen;
use crate::ds::transformation::{CameraInfo, ObjectTransform, ProjectionInfo};
use crate::ds::{model::Model, wgpu_resource::RendererState};
use crate::render::{render_pass, render_payload};
use crate::utils::render_setup_factory;
use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use winit::platform::web::EventLoopExtWebSys;

const DEMO_VERT_SHADER_PATH: &str = "./src/shaders/flat_color.wgsl";
const DEMO_FRAG_SHADER_PATH: &str = "./src/shaders/flat_color.wgsl";

pub struct AppState {
    window: Arc<Window>,
    renderer_state: RendererState,
}

impl AppState {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();
        let renderer_state = render_setup_factory::create_render_setup_raster_standard(
            DEMO_VERT_SHADER_PATH,
            DEMO_FRAG_SHADER_PATH,
            Some(Screen {
                window: window.clone(),
                window_inner_width: size.width,
                window_inner_height: size.height,
            }),
        )
        .await?;

        Ok(Self {
            window,
            renderer_state,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            let surface_state = self
                .renderer_state
                .wgpu_object
                .surface_state
                .as_mut()
                .expect("Surface state should be initialized before resizing.");

            surface_state.config.width = width;
            surface_state.config.height = height;

            surface_state.surface.configure(
                &self.renderer_state.wgpu_object.device,
                &surface_state.config,
            );

            surface_state.is_surface_configured = true;
        }
    }

    pub fn render(
        &mut self,
        render_payload: &render_payload::RenderPayload,
        draw_indices: core::ops::Range<u32>,
        depth_attachment_texture: &wgpu::Texture,
    ) -> anyhow::Result<()> {
        self.window.request_redraw();
        if !self
            .renderer_state
            .wgpu_object
            .surface_state
            .as_ref()
            .expect("Surface state should be initialized before rendering.")
            .is_surface_configured
        {
            return Ok(());
        }

        let transform_bind_group: &wgpu::BindGroup = &render_payload.transform_bind_group;
        let mat_light_bind_group: &wgpu::BindGroup = &render_payload.mat_light_bind_group;
        let vertex_buffer: &wgpu::Buffer = &render_payload.vertex_buffer;
        let index_buffer: &wgpu::Buffer = &render_payload.index_buffer;

        let wgpu_obj = &mut self.renderer_state.wgpu_object;
        let render_pipeline = &self.renderer_state.render_pipeline;
        let surface_state = wgpu_obj.surface_state.as_mut().unwrap();

        let clear_color = wgpu::Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };

        let _submission_index = render_pass::render_to_screen(
            &wgpu_obj.device,
            &wgpu_obj.queue,
            &surface_state.surface,
            &surface_state.config,
            &render_pipeline,
            transform_bind_group,
            mat_light_bind_group,
            vertex_buffer,
            index_buffer,
            draw_indices,
            clear_color,
            depth_attachment_texture,
        );
        Ok(())
    }

    fn update(&mut self) {}

    fn handle_key(&self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {}
        }
    }
}

pub struct App {
    #[cfg(target_arch = "wasm32")]
    proxy: Option<winit::event_loop::EventLoopProxy<AppState>>,
    state: Option<AppState>,
    model_list: Vec<Model>,
}

impl App {
    pub fn new(
        #[cfg(target_arch = "wasm32")] event_loop: &EventLoop<AppState>,
        initial_model: Model,
    ) -> Self {
        #[cfg(target_arch = "wasm32")]
        let proxy = Some(event_loop.create_proxy());

        Self {
            state: None,
            model_list: vec![initial_model],
            #[cfg(target_arch = "wasm32")]
            proxy,
        }
    }
}

impl ApplicationHandler<AppState> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes();

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;

            const CANVAS_ID: &str = "canvas";

            let window = wgpu::web_sys::window().unwrap_throw();
            let document = window.document().unwrap_throw();
            let canvas = document.get_element_by_id(CANVAS_ID).unwrap_throw();
            let html_canvas_element = canvas.unchecked_into();
            window_attributes = window_attributes.with_canvas(Some(html_canvas_element));
        }

        let window = Arc::new(
            event_loop
                .create_window(window_attributes)
                .expect("Failed to create window"),
        );

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.state = Some(
                pollster::block_on(AppState::new(window))
                    .expect("Failed to create window in WASM 32"),
            );
        }

        #[cfg(target_arch = "wasm32")]
        {
            // Run the future asynchronously and use the
            // proxy to send the results to the event loop
            if let Some(proxy) = self.proxy.take() {
                wasm_bindgen_futures::spawn_local(async move {
                    assert!(
                        proxy
                            .send_event(
                                AppState::new(window)
                                    .await
                                    .expect("Unable to create canvas!!!")
                            )
                            .is_ok()
                    )
                });
            }
        }
    }

    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: AppState) {
        #[cfg(target_arch = "wasm32")]
        {
            event.window.request_redraw();
            event.resize(
                event.window.inner_size().width,
                event.window.inner_size().height,
            );
        }
        self.state = Some(event);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let state: &mut AppState = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                state.update();

                let renderer_state = &mut state.renderer_state;
                let wgpu_obj = &mut renderer_state.wgpu_object;
                let surface_state = wgpu_obj
                    .surface_state
                    .as_mut()
                    .expect("Surface state should be initialized before resizing.");

                let current_model = &mut self.model_list[0]; // TODO: Unsafe, for temporary testing

                let face_len = current_model.mesh().faces().len();
                let output_width = surface_state.config.width;
                let output_height = surface_state.config.height;

                let device = &mut wgpu_obj.device;

                let depth_attachment_texture_descriptor = wgpu::TextureDescriptor {
                    label: Some("Output Depth Texture"),
                    size: wgpu::Extent3d {
                        width: output_width,
                        height: output_height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Depth32Float,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                };

                let depth_attachment_texture: wgpu::Texture =
                    device.create_texture(&depth_attachment_texture_descriptor);

                let object_transform = ObjectTransform::default();

                let camera_info = CameraInfo {
                    position: glam::Vec3::new(5.0, 5.0, 5.0),
                    look_at: glam::Vec3::new(0.0, 0.0, 0.0),
                    up: glam::Vec3::new(0.0, 1.0, 0.0),
                    fov: 45.0,
                };

                let projection_info = ProjectionInfo {
                    near: 1.0,
                    far: 1000.0,
                };

                let render_payload = render_payload::create_initial_render_payload(
                    device,
                    current_model,
                    &renderer_state.bind_group_layouts,
                    &object_transform,
                    &camera_info,
                    &projection_info,
                    output_width,
                    output_height,
                );

                match state.render(
                    &render_payload,
                    0..(face_len * 3) as u32,
                    &depth_attachment_texture,
                ) {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("{e}");
                        event_loop.exit();
                    }
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => state.handle_key(event_loop, code, key_state.is_pressed()),
            _ => {}
        }
    }
}

pub fn run(initial_model: Model) -> anyhow::Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Info).unwrap_throw();
    }

    let event_loop = EventLoop::with_user_event().build()?;
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut app = App::new(initial_model);
        event_loop.run_app(&mut app)?;
    }
    #[cfg(target_arch = "wasm32")]
    {
        let app = App::new(&event_loop, initial_model);
        event_loop.spawn_app(app);
    }

    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run_web(initial_model: Model) -> Result<(), wasm_bindgen::JsValue> {
    console_error_panic_hook::set_once();
    run(initial_model).unwrap_throw();

    Ok(())
}
