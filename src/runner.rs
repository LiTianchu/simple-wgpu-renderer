use crate::constants;
use crate::ds::model::{MaterialStore, TextureStore};
use crate::ds::transformation::{CameraInfo, ObjectTransform, ProjectionInfo};
use crate::ds::viewer::{EguiFrame, ViewerState};
use crate::ds::{
    model::Scene,
    wgpu_resource::{RendererState, SceneBindGroupLayoutSet, WgpuObject},
};
use crate::io::model_loader;
use crate::render::{factory::render_setup_factory, render_pass, render_payload};
use glam::Vec3;
use std::path::PathBuf;
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
    pub async fn new(
        window: Arc<Window>,
        wgpu_object: &WgpuObject,
        scene_bind_group_layouts: &SceneBindGroupLayoutSet,
        material_bind_group_layout: &wgpu::BindGroupLayout,
        texture_sampler_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> anyhow::Result<Self> {
        let size = window.inner_size();
        let renderer_state = render_setup_factory::create_render_setup_raster_standard(
            wgpu_object,
            constants::TEXTURED_VERT_SHADER_PATH,
            constants::TEXTURED_FRAG_SHADER_PATH,
            (size.width, size.height),
            scene_bind_group_layouts,
            material_bind_group_layout,
            texture_sampler_bind_group_layout,
        )
        .await?;

        // ========= EGUI Setup =========
        let egui_ctx = egui::Context::default();
        let max_texture_side = wgpu_object.device.limits().max_texture_dimension_2d as usize;

        let surface_format = wgpu_object
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

        let egui_renderer =
            egui_wgpu::Renderer::new(&wgpu_object.device, surface_format, Default::default());
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

    pub fn resize(&mut self, width: u32, height: u32, wgpu_object: &mut WgpuObject) {
        if width > 0 && height > 0 {
            let surface_state = wgpu_object
                .surface_state
                .as_mut()
                .expect("Surface state should be initialized before resizing.");

            surface_state.config.width = width;
            surface_state.config.height = height;

            surface_state
                .surface
                .configure(&wgpu_object.device, &surface_state.config);

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

            let depth_attachment_texture: wgpu::Texture = wgpu_object
                .device
                .create_texture(&depth_attachment_texture_descriptor);

            self.renderer_state.depth_attachment_texture = depth_attachment_texture;
        }
    }

    pub fn render(
        &mut self,
        scene: &Scene,
        wgpu_object: &WgpuObject,
        render_payload: &render_payload::RenderPayload,
        material_store: &MaterialStore,
        texture_store: &mut TextureStore,
        egui_frame: &mut EguiFrame,
    ) -> anyhow::Result<()> {
        self.window.request_redraw();
        if !wgpu_object
            .surface_state
            .as_ref()
            .expect("Surface state should be initialized before rendering.")
            .is_surface_configured
        {
            return Ok(());
        }

        let transform_bind_group: &wgpu::BindGroup = &render_payload.transform_bind_group;
        let light_bind_group: &wgpu::BindGroup = &render_payload.light_bind_group;

        let render_pipeline = &self.renderer_state.render_pipeline;
        let surface_state = wgpu_object.surface_state.as_ref().unwrap();

        let clear_color = wgpu::Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };

        let color_output_surface_texture = match surface_state.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => {
                // skip this frame
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                surface_state
                    .surface
                    .configure(&wgpu_object.device, &surface_state.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                panic!("Device is lost during window render!")
            }
        };

        let _submission_index = render_pass::render_to_screen(
            &wgpu_object.device,
            &wgpu_object.queue,
            &render_pipeline,
            transform_bind_group,
            light_bind_group,
            clear_color,
            &render_payload.vertex_buffer,
            &render_payload.index_buffer,
            material_store,
            texture_store,
            scene,
            color_output_surface_texture,
            &self.renderer_state.depth_attachment_texture,
            &mut self.egui_renderer,
            egui_frame.paint_jobs.as_slice(),
            &mut egui_frame.textures_delta,
            &egui_frame.screen_descriptor,
        )?;

        Ok(())
    }

    fn update(&mut self) {}

    fn handle_key(&self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {}
        }
    }

    fn create_egui_frame(
        &mut self,
        wgpu_object: &WgpuObject,
        event_loop: &ActiveEventLoop,
    ) -> EguiFrame {
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

        let config = &wgpu_object
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

struct AppResources {
    wgpu_object: WgpuObject,
    scene: Scene,
    material_store: MaterialStore,
    texture_store: TextureStore,
    scene_bind_group_layouts: SceneBindGroupLayoutSet,
}

pub struct App {
    state: Option<AppState>,
    resources: Option<AppResources>,
    model_paths: Vec<PathBuf>,
    initialization_error: Option<anyhow::Error>,
}

impl App {
    pub fn new(model_paths: Vec<PathBuf>) -> Self {
        Self {
            state: None,
            resources: None,
            model_paths,
            initialization_error: None,
        }
    }

    async fn initialize(
        window: Arc<Window>,
        model_paths: Vec<PathBuf>,
    ) -> anyhow::Result<(AppState, AppResources)> {
        let wgpu_object = WgpuObject::on_screen(window.clone()).await;
        let scene_bind_group_layouts = SceneBindGroupLayoutSet::new(&wgpu_object.device);
        let mut material_store = MaterialStore::new(&wgpu_object.device);
        let texture_store = TextureStore::new(&wgpu_object.device, &wgpu_object.queue);
        let scene = model_loader::load_obj_models_to_scene(
            model_paths,
            &mut material_store,
            &wgpu_object.device,
        )
        .ok_or_else(|| anyhow::anyhow!("Failed to load the requested models"))?;

        let state = AppState::new(
            window,
            &wgpu_object,
            &scene_bind_group_layouts,
            material_store.material_bind_group_layout(),
            texture_store.texture_sampler_bind_group_layout(),
        )
        .await?;

        Ok((
            state,
            AppResources {
                wgpu_object,
                scene,
                material_store: material_store,
                texture_store: texture_store,
                scene_bind_group_layouts,
            },
        ))
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        let window_attributes = Window::default_attributes()
            .with_title("My Renderer")
            .with_inner_size(constants::WINDOW_PHYSICAL_SIZE);

        let window = match event_loop.create_window(window_attributes) {
            Ok(window) => Arc::new(window),
            Err(error) => {
                self.initialization_error = Some(error.into());
                event_loop.exit();
                return;
            }
        };

        match pollster::block_on(Self::initialize(window, self.model_paths.clone())) {
            Ok((state, resources)) => {
                self.state = Some(state);
                self.resources = Some(resources);
            }
            Err(error) => {
                self.initialization_error = Some(error);
                event_loop.exit();
            }
        }
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
        let resources = match &mut self.resources {
            Some(resources) => resources,
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
            WindowEvent::Resized(size) => {
                state.resize(size.width, size.height, &mut resources.wgpu_object)
            }
            WindowEvent::RedrawRequested => {
                state.update();

                let mut egui_frame = state.create_egui_frame(&resources.wgpu_object, event_loop);

                let surface_state = resources
                    .wgpu_object
                    .surface_state
                    .as_ref()
                    .expect("Surface state should be initialized before resizing.");

                let output_width = surface_state.config.width;
                let output_height = surface_state.config.height;
                let scene_bind_group_layouts = &resources.scene_bind_group_layouts;
                let device = &resources.wgpu_object.device;

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

                let render_payload = render_payload::create_standard_render_payload(
                    device,
                    scene_bind_group_layouts,
                    &object_transform,
                    &camera_info,
                    &projection_info,
                    output_width,
                    output_height,
                );

                match state.render(
                    &resources.scene,
                    &resources.wgpu_object,
                    &render_payload,
                    &resources.material_store,
                    &mut resources.texture_store,
                    &mut egui_frame,
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

pub fn run(model_paths: Vec<PathBuf>) -> anyhow::Result<()> {
    env_logger::init();

    let event_loop = EventLoop::new()?;
    let mut app = App::new(model_paths);
    event_loop.run_app(&mut app)?;

    if let Some(error) = app.initialization_error {
        return Err(error);
    }

    Ok(())
}
