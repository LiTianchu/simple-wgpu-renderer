use crate::constants;
use crate::ds::model::{MaterialStore, TextureObject, TextureStore};
use crate::ds::transformation::{CameraInfo, ObjectTransform, ProjectionInfo};
use crate::ds::viewer::{EguiFrame, Screen, ViewerState};
use crate::ds::{
    model::{Material, Scene},
    wgpu_resource::RendererState,
};
use crate::render::{factory::render_setup_factory, render_pass, render_payload};
use glam::Vec3;
use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

pub struct AppState {
    window: Arc<Window>,
    renderer_state: RendererState,
    egui_ctx: egui::Context,
    egui_winit: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
    viewer_state: ViewerState,
}

impl AppState {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();
        let renderer_state = render_setup_factory::create_render_setup_raster_standard(
            constants::TEXTURED_VERT_SHADER_PATH,
            constants::TEXTURED_FRAG_SHADER_PATH,
            Some(Screen {
                window: window.clone(),
                window_inner_width: size.width,
                window_inner_height: size.height,
            }),
            (size.width, size.height),
        )
        .await?;

        // ========= EGUI Setup =========
        let egui_ctx = egui::Context::default();
        let max_texture_side = renderer_state
            .wgpu_object
            .device
            .limits()
            .max_texture_dimension_2d as usize;

        let surface_format = renderer_state
            .wgpu_object
            .surface_state
            .as_ref()
            .expect("Surface state should be initialized before creating egui renderer.")
            .config
            .format;

        let egui_winit = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            window.as_ref(),
            Some(window.scale_factor() as f32),
            window.theme(),
            Some(max_texture_side),
        );

        let egui_renderer = egui_wgpu::Renderer::new(
            &renderer_state.wgpu_object.device,
            surface_format,
            Default::default(),
        );
        // ========= End EGUI Setup =========

        let viewer_state = ViewerState {
            model_rotation_euler_deg: glam::Vec3::new(0.0, 0.0, 0.0),
            model_scale_uniform: 1.0,
            cam_elevation_deg: 45.0,
            cam_radius: 5.0,
            cam_fov_deg: 45.0,
        };

        Ok(Self {
            window,
            renderer_state,
            egui_ctx,
            egui_winit,
            egui_renderer,
            viewer_state,
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

            // recreate depth texture with the new window size
            let depth_attachment_texture_descriptor = wgpu::TextureDescriptor {
                label: Some("Output Depth Texture"),
                size: wgpu::Extent3d {
                    width: width,
                    height: height,
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
                self.renderer_state.wgpu_object.device.create_texture(&depth_attachment_texture_descriptor);

            self.renderer_state.depth_attachment_texture = depth_attachment_texture;
        }
    }

    pub fn render(
        &mut self,
        render_payload: &render_payload::RenderPayload,
        draw_indices: core::ops::Range<u32>,
        egui_frame: &mut EguiFrame,
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
        let light_bind_group: &wgpu::BindGroup = &render_payload.light_bind_group;
        let mat_bind_group: &wgpu::BindGroup = &render_payload.mat_bind_group;
        let texture_sampler_bind_group: Option<&wgpu::BindGroup> =
            render_payload.texture_sampler_bind_group.as_ref();
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
            light_bind_group,
            mat_bind_group,
            texture_sampler_bind_group,
            vertex_buffer,
            index_buffer,
            draw_indices,
            clear_color,
            &self.renderer_state.depth_attachment_texture,
            &mut self.egui_renderer,
            egui_frame.paint_jobs.as_slice(),
            &mut egui_frame.textures_delta,
            &egui_frame.screen_descriptor,
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

    fn create_egui_frame(&mut self, event_loop: &ActiveEventLoop) -> EguiFrame {
        let raw_input = self.egui_winit.take_egui_input(&self.window);

        let output: egui::FullOutput = self.egui_ctx.run_ui(raw_input, |ui| {
            ui.heading("Renderer Controls");
            ui.add(
                egui::Slider::new(
                    &mut self.viewer_state.model_rotation_euler_deg.x,
                    0.0..=360.0,
                )
                .text("Model Rotation X"),
            );
            ui.add(
                egui::Slider::new(
                    &mut self.viewer_state.model_rotation_euler_deg.y,
                    0.0..=360.0,
                )
                .text("Model Rotation Y"),
            );
            ui.add(
                egui::Slider::new(
                    &mut self.viewer_state.model_rotation_euler_deg.z,
                    0.0..=360.0,
                )
                .text("Model Rotation Z"),
            );
            ui.add(
                egui::Slider::new(&mut self.viewer_state.model_scale_uniform, 0.0..=10.0)
                    .text("Model Uniform Scale"),
            );
            ui.add(
                egui::Slider::new(&mut self.viewer_state.cam_elevation_deg, -90.0..=90.0)
                    .text("Camera Elevation Angle"),
            );
            ui.add(
                egui::Slider::new(&mut self.viewer_state.cam_radius, 0.01..=20.0)
                    .text("Camera Orbit Radius"),
            );
            ui.add(
                egui::Slider::new(&mut self.viewer_state.cam_fov_deg, 0.01..=120.0)
                    .text("Camera Field of View (FOV)"),
            );
        });

        // destruct output
        let egui::FullOutput {
            platform_output,
            textures_delta,
            shapes,
            pixels_per_point,
            ..
        } = output;

        self.egui_winit.handle_platform_output_with_event_loop(
            &self.window,
            event_loop,
            platform_output,
        );

        let paint_jobs = self.egui_ctx.tessellate(shapes, pixels_per_point);

        let config = &self
            .renderer_state
            .wgpu_object
            .surface_state
            .as_ref()
            .expect("Surface state should be initialized before creating egui frame.")
            .config;

        EguiFrame {
            paint_jobs,
            textures_delta,
            screen_descriptor: egui_wgpu::ScreenDescriptor {
                size_in_pixels: [config.width, config.height],
                pixels_per_point: pixels_per_point,
            },
        }
    }
}

pub struct App {
    state: Option<AppState>,
    scene: Scene,
    material_store: MaterialStore,
    texture_store: TextureStore,
}

impl App {
    pub fn new(
        initial_scene: Scene,
        material_store: MaterialStore,
        texture_store: TextureStore,
    ) -> Self {
        Self {
            state: None,
            scene: initial_scene,
            material_store: material_store,
            texture_store: texture_store,
        }
    }

    pub fn material_store(&self) -> &MaterialStore {
        &self.material_store
    }

    pub fn material_store_mut(&mut self) -> &mut MaterialStore {
        &mut self.material_store
    }

    pub fn texture_store(&self) -> &TextureStore {
        &self.texture_store
    }

    pub fn texture_store_mut(&mut self) -> &mut TextureStore {
        &mut self.texture_store
    }

    pub fn insert_material_to_store(&mut self, material_key: String, material: Material) {
        self.material_store.insert_material(material_key, material);
    }
}

impl ApplicationHandler<AppState> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes()
            .with_title("My Renderer")
            .with_inner_size(constants::WINDOW_PHYSICAL_SIZE);

        let window = Arc::new(
            event_loop
                .create_window(window_attributes)
                .expect("Failed to create window"),
        );

        {
            self.state =
                Some(pollster::block_on(AppState::new(window)).expect(
                    "Failed to block the thread and create the AppState for the application.",
                ));
        }
    }

    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: AppState) {
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

        if matches!(&event, WindowEvent::CloseRequested) {
            event_loop.exit();
        }

        // forward window event to egui
        let egui_response = state
            .egui_winit
            .on_window_event(state.window.as_ref(), &event);

        // if need ui refresh, request redraw
        if egui_response.repaint {
            state.window.request_redraw();
        }

        // prevent ui click-through
        if egui_response.consumed {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                state.update();

                let mut egui_frame = state.create_egui_frame(event_loop);

                let renderer_state = &mut state.renderer_state;
                let wgpu_obj = &mut renderer_state.wgpu_object;
                let surface_state = wgpu_obj
                    .surface_state
                    .as_mut()
                    .expect("Surface state should be initialized before resizing.");

                let bind_group_layouts = &renderer_state.bind_group_layouts;
                let device = &mut wgpu_obj.device;
                let queue = &wgpu_obj.queue;

                let output_width = surface_state.config.width;
                let output_height = surface_state.config.height;

                let mut object_transform = ObjectTransform::default();

                object_transform.set_rotation_euler(
                    state
                        .viewer_state
                        .model_rotation_euler_deg
                        .map(|deg| deg.to_radians()),
                );

                object_transform.set_scale_uniform(state.viewer_state.model_scale_uniform);

                // y = radius * sin(elevation)
                let camera_info = CameraInfo {
                    fov: state.viewer_state.cam_fov_deg.to_radians(),
                    position: Vec3::new(
                        0.0,
                        state.viewer_state.cam_radius
                            * state.viewer_state.cam_elevation_deg.to_radians().sin(),
                        state.viewer_state.cam_radius,
                    ),
                    ..Default::default()
                };

                let projection_info = ProjectionInfo::default();

                // TODO: Support rendering multiple models in the scene
                let model = self
                    .scene
                    .models()
                    .first()
                    .ok_or_else(|| anyhow::anyhow!("Scene has no models to render!"))
                    .unwrap();

                let face_len = model.face_count();
                println!(
                    "Rendering model: {}\n  Vert count: {}\n  Face count: {}",
                    model.file_path(),
                    model.vert_count(),
                    model.face_count()
                );

                let temp_mesh = model
                    .meshes()
                    .first()
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "Model has no meshes to render! Model file path: {}",
                            model.file_path()
                        )
                    })
                    .unwrap();

                let material = temp_mesh
                    .mat_key()
                    .and_then(|mat_key| self.material_store.get_material(mat_key));

                let mut texture_obj: Option<&TextureObject> = None;

                if let Some(mat) = material {
                    if let Some(p) = mat.texture_set.diffuse_map_path.as_ref() {
                        let full_path = format!("{}/{}", model.model_dir_path(), p);

                        let tex_option = self.texture_store.get_or_load_texture(
                            &device,
                            &queue,
                            full_path.clone(),
                            wgpu::TextureFormat::Rgba8UnormSrgb,
                            "Test Texture",
                        );

                        match tex_option {
                            Some(tex) => {
                                println!("Found diffuse texture at: {}", full_path);
                                texture_obj = Some(tex);
                            }
                            None => {
                                println!("No diffuse texture found at: {}", full_path);
                            }
                        }
                    }
                }
                let render_payload = render_payload::create_standard_render_payload(
                    device,
                    model,
                    bind_group_layouts,
                    &object_transform,
                    &camera_info,
                    &projection_info,
                    texture_obj,
                    output_width,
                    output_height,
                );

                match state.render(&render_payload, 0..(face_len * 3) as u32, &mut egui_frame) {
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

pub fn run(
    initial_scene: Scene,
    material_store: MaterialStore,
    texture_store: TextureStore,
) -> anyhow::Result<()> {
    env_logger::init();

    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = App::new(initial_scene, material_store, texture_store);
    event_loop.run_app(&mut app)?;

    Ok(())
}
