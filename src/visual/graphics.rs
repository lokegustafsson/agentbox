use crate::{visual::solids_renderer::SolidsRenderer, Solid};
use anyhow::*;
use cgmath::{prelude::*, Matrix4, Vector2};
use log::info;
use std::mem;
use wgpu::*;
use winit::{dpi::PhysicalSize, window::Window};

const TEXTURE_FORMAT: TextureFormat = TextureFormat::Bgra8UnormSrgb;

pub const PUSH_CONSTANT_RANGE: PushConstantRange = PushConstantRange {
    stages: ShaderStage::FRAGMENT,
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
    queue: Queue,
    device: Device,
    surface: Surface,
    swap_chain: SwapChain,

    solids_renderer: SolidsRenderer,

    push_constants: PushConstants,
}
impl Graphics {
    pub async fn initialize(window: &Window) -> Result<Self> {
        let instance = Instance::new(BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = build_adapter(&instance, &surface).await?;
        let (device, queue) = build_device_and_queue(&adapter).await?;

        info!("Found and acquired adapter:\n{:?}", adapter.get_info());

        let window_size: (u32, u32) = window.inner_size().into();
        let swap_chain = build_swap_chain(&device, &surface, window_size);

        let solids_renderer = SolidsRenderer::new(&device);

        let push_constants = PushConstants::new(Vector2::from(window_size).cast().unwrap());

        Ok(Self {
            queue,
            device,
            surface,
            swap_chain,
            solids_renderer,
            push_constants,
        })
    }
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        let size: (f32, f32) = (new_size.width as f32, new_size.height as f32);
        self.push_constants.window_size = Vector2::from(size);
        self.swap_chain = build_swap_chain(&self.device, &self.surface, new_size.into());
    }
    pub fn update_world(&mut self, solids: Vec<Solid>) {
        self.solids_renderer.update(solids);
        self.solids_renderer.push_to_gpu_buffers(&self.queue);
    }
    pub fn render(&mut self, camera_to_world: Matrix4<f32>) {
        self.push_constants.camera_to_world = camera_to_world;

        // Render
        {
            let swap_chain_frame = self.swap_chain.get_current_frame().unwrap();
            let mut encoder = self
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("The render pass command encoder"),
                });
            {
                let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("The render pass"),
                    color_attachments: &[RenderPassColorAttachmentDescriptor {
                        attachment: &swap_chain_frame.output.view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Load,
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: None,
                });
                self.solids_renderer.render(&mut pass, self.push_constants);
            }

            self.queue.submit(std::iter::once(encoder.finish()));
        }
    }
}

fn build_swap_chain(device: &Device, surface: &Surface, (width, height): (u32, u32)) -> SwapChain {
    device.create_swap_chain(
        surface,
        &SwapChainDescriptor {
            usage: TextureUsage::RENDER_ATTACHMENT,
            format: TEXTURE_FORMAT,
            width,
            height,
            present_mode: PresentMode::Fifo,
        },
    )
}

async fn build_adapter(instance: &Instance, surface: &Surface) -> Result<Adapter> {
    instance
        .request_adapter(&RequestAdapterOptionsBase {
            power_preference: PowerPreference::default(),
            compatible_surface: Some(&surface),
        })
        .await
        .context("Failed to acquire adapter")
}

async fn build_device_and_queue(adapter: &Adapter) -> Result<(Device, Queue)> {
    adapter
        .request_device(
            &DeviceDescriptor {
                label: Some("The device"),
                features: Features::PUSH_CONSTANTS,
                limits: Limits {
                    max_push_constant_size: mem::size_of::<PushConstants>() as u32,
                    ..Limits::default()
                },
            },
            None, // Trace path
        )
        .await
        .context("Failed to acquire device")
}
