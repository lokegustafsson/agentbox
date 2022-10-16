use crate::{visual::solids_renderer::SolidsRenderer, Solid};
use cgmath::{prelude::*, Matrix4, Vector2};
use log::info;
use std::mem;
use winit::{dpi::PhysicalSize, window::Window};

const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

pub const PUSH_CONSTANT_RANGE: wgpu::PushConstantRange = wgpu::PushConstantRange {
    stages: wgpu::ShaderStages::FRAGMENT,
    range: 0..mem::size_of::<PushConstants>() as u32,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PushConstants {
    pub(self) camera_to_world: Matrix4<f32>,
    pub(self) window_size: Vector2<f32>,
    _padding: [u32; 2],
}
impl PushConstants {
    pub fn new(window_size: Vector2<f32>) -> Self {
        Self {
            camera_to_world: Matrix4::zero(), // WRONG
            window_size,
            _padding: [0, 0],
        }
    }
}
unsafe impl bytemuck::Pod for PushConstants {}
unsafe impl bytemuck::Zeroable for PushConstants {}

// TODO Use push constants instead of uniforms
pub struct Graphics {
    queue: wgpu::Queue,
    device: wgpu::Device,
    surface: wgpu::Surface,
    window_size: (u32, u32),

    solids_renderer: SolidsRenderer,

    push_constants: PushConstants,
}
impl Graphics {
    pub async fn initialize(window: &Window) -> Self {
        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = build_adapter(&instance, &surface).await;
        let (device, queue) = build_device_and_queue(&adapter).await;

        info!("Found and acquired adapter:\n{:?}", adapter.get_info());

        let window_size: (u32, u32) = window.inner_size().into();
        configure_surface(&device, &surface, window_size);

        let solids_renderer = SolidsRenderer::new(&device);

        let push_constants = PushConstants::new(Vector2::from(window_size).cast().unwrap());

        Self {
            queue,
            device,
            surface,
            window_size,
            solids_renderer,
            push_constants,
        }
    }
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        let size: (f32, f32) = (new_size.width as f32, new_size.height as f32);
        self.push_constants.window_size = Vector2::from(size);
        self.window_size = (new_size.width, new_size.height);
        configure_surface(&self.device, &self.surface, new_size.into());
    }
    pub fn update_world(&mut self, solids: Vec<Solid>) {
        self.solids_renderer.update(solids);
        self.solids_renderer.push_to_gpu_buffers(&self.queue);
    }
    pub fn render(&mut self, camera_to_world: Matrix4<f32>) {
        self.push_constants.camera_to_world = camera_to_world;

        let surface_texture = self
            .surface
            .get_current_texture()
            .or_else(|error| {
                log::debug!("retrying `wgpu::Surface::get_current_texture` once after: {error:?}");
                configure_surface(&self.device, &self.surface, self.window_size);
                self.surface.get_current_texture()})
            .unwrap();
        let surface_texture_view =
            surface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor {
                    label: Some("The render pass texture view"),
                    format: None, // REALLY?
                    dimension: None,
                    aspect: wgpu::TextureAspect::All,
                    base_mip_level: 0,
                    mip_level_count: None,
                    base_array_layer: 0,
                    array_layer_count: None,
                });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("The render pass command encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("The render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            self.solids_renderer.render(&mut pass, self.push_constants);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();
    }
}

fn configure_surface(device: &wgpu::Device, surface: &wgpu::Surface, (width, height): (u32, u32)) {
    surface.configure(
        &device,
        &wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: TEXTURE_FORMAT,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        },
    )
}

async fn build_adapter(instance: &wgpu::Instance, surface: &wgpu::Surface) -> wgpu::Adapter {
    instance
        .request_adapter(&wgpu::RequestAdapterOptionsBase {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .unwrap_or_else(|| panic!("Failed requesting adapter"))
}

async fn build_device_and_queue(adapter: &wgpu::Adapter) -> (wgpu::Device, wgpu::Queue) {
    adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("The device"),
                features: wgpu::Features::PUSH_CONSTANTS,
                limits: wgpu::Limits {
                    max_push_constant_size: mem::size_of::<PushConstants>() as u32,
                    ..wgpu::Limits::default()
                },
            },
            None, // Trace path
        )
        .await
        .unwrap()
}
