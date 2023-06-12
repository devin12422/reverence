// use async_trait::async_trait;
use std::future::Future;
// use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use wgpu::{util::DeviceExt, Surface};
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
pub trait WindowAbstractor // where
{
    fn get_size(&self) -> [u32; 2];
}
pub trait RustWindowAbstractor: WindowAbstractor + 'static {
    type Window: HasRawWindowHandle + HasRawDisplayHandle;
    fn get_window(&self) -> &Window;
}
pub trait GPUAbstractor
where
    Self: Send,
{
    fn resize(&mut self, new_size: impl Into<[u32; 2]>);
}
#[async_trait]
pub trait RendererAbstractor<WindowInterface, GPUInterface>
where
    WindowInterface: WindowAbstractor,
    GPUInterface: GPUAbstractor,
{
    async fn render(&mut self);
}
pub trait SystemBackend {
    fn get_time() -> SystemTime;
}
// pub trait Renderer<GPUInterface>
// where
//     Self: HasCreateInfo + Send + 'static,
//     GPUInterface: GPUAbstractor,
// {
//     // async fn new(window: Window) -> Self;
//     fn render<'a>(
//         &'a mut self,
//     ) -> impl Future<Output = Result<(), wgpu::SurfaceError>> + Send + 'a;
//     fn get_window(&self) -> &Window;
// }
pub struct WGPUInterface
where
    Self: Send + 'static,
{
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
}
use std::sync::Arc;
impl WGPUInterface {
    pub fn new<W>(
        window: Arc<W>,
        size: impl Into<[u32; 2]>,
    ) -> impl Future<Output = Self> + Send + 'static
    where
        W: HasRawWindowHandle + HasRawDisplayHandle + 'static,
    {
        let size = size.into();
        let size = {
            winit::dpi::PhysicalSize {
                width: size[0],
                height: size[1],
            }
        };
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = unsafe { instance.create_surface(window.as_ref()) }.unwrap();
        async move {
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::default(),
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })
                .await
                .unwrap();
            println!("{:?}", adapter.features());
            let (device, queue) = adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        features: wgpu::Features::empty(),
                        limits: if cfg!(target_arch = "wasm32") {
                            wgpu::Limits::downlevel_webgl2_defaults()
                        } else {
                            wgpu::Limits::default()
                        },
                        label: None,
                    },
                    None,
                )
                .await
                .unwrap();
            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface.get_capabilities(&adapter).formats[0],
                view_formats: vec![], //surface.get_capabilities(&adapter).formats,
                width: size.width,
                height: size.height,
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
            };
            surface.configure(&device, &config);
            Self {
                surface,
                device,
                queue,
                config,
                size,
            }
        }
    }
}
impl GPUAbstractor for WGPUInterface {
    fn resize(&mut self, new_size: impl Into<[u32; 2]>) {
        let new_size = new_size.into();
        println!("{:?}", new_size);
        if new_size[0] > 0 && new_size[1] > 0 {
            self.size = PhysicalSize {
                width: new_size[0],
                height: new_size[1],
            };
            self.config.width = self.size.width;
            self.config.height = self.size.height;
            self.surface.configure(&self.device, &self.config);
            // self.renderer.resize(new_size);
        }
    }
}
