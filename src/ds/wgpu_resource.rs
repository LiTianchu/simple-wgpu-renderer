use crate::ds::screen::Screen;
use std::path::PathBuf;

#[derive(Debug)]
pub struct WgpuObject {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_state: Option<SurfaceState>,
}

impl WgpuObject {
    pub async fn off_screen() -> Self {
        let wgpu_instance_descriptor: wgpu::InstanceDescriptor = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            backend_options: Default::default(),
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            display: None,
        };

        let wgpu_instance = wgpu::Instance::new(wgpu_instance_descriptor);

        let request_adator_options = wgpu::RequestAdapterOptions {
            power_preference: Default::default(),
            // None for off-screen rendering, need to pass in &surface if render on screen
            compatible_surface: None,
            force_fallback_adapter: false,
            apply_limit_buckets: true,
        };

        let wgpu_adapter: wgpu::Adapter = wgpu_instance
            .request_adapter(&request_adator_options)
            .await
            .expect("Failed to request WGPU adapter when initializing WGPU state for off_screen rendering.");

        let device_required_features = wgpu::Features::empty();
        let device_exp_features = wgpu::ExperimentalFeatures::disabled();

        let device_descriptor = wgpu::DeviceDescriptor {
            label: Some("Image Export Device Descriptor."),
            required_features: device_required_features,
            experimental_features: device_exp_features,
            required_limits: wgpu::Limits::defaults(),
            memory_hints: wgpu::MemoryHints::default(),
            trace: wgpu::Trace::Off,
        };

        let (device, queue) = wgpu_adapter
            .request_device(&device_descriptor)
            .await
            .expect("Failed to create WGPU device and queue when initializing WGPU state for off_screen rendering.");

        Self {
            instance: wgpu_instance,
            adapter: wgpu_adapter,
            device,
            queue,
            surface_state: None,
        }
    }

    pub async fn on_screen(screen: Screen) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });

        let surface = instance.create_surface(screen.window.clone()).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: true,
            })
            .await.expect("Failed to request WGPU adapter when initializing WGPU state for on_screen rendering.");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await.expect("Failed to create WGPU device and queue when initializing WGPU state for on_screen rendering.");

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: screen.window_inner_width,
            height: screen.window_inner_height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: wgpu::SurfaceColorSpace::Auto,
        };

        Self {
            instance: instance,
            adapter: adapter,
            device,
            queue,
            surface_state: Some(SurfaceState {
                surface,
                config,
                is_surface_configured: false,
            }),
        }
    }
}

#[derive(Debug)]
pub struct RendererState {
    pub wgpu_object: WgpuObject,
    pub render_pipeline: wgpu::RenderPipeline,
    pub bind_group_layouts: BindGroupLayoutState,
    pub frag_texture_format: wgpu::TextureFormat,
    pub vert_shader_path: PathBuf,
    pub frag_shader_path: PathBuf,
}

#[derive(Debug)]
pub struct SurfaceState {
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
    pub is_surface_configured: bool,
}

#[derive(Debug, Clone)]
pub struct BindGroupLayoutState {
    pub transform_bind_group_layout: wgpu::BindGroupLayout,
    pub mat_light_bind_group_layout: wgpu::BindGroupLayout,
}
