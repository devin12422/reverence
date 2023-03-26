#![feature(type_alias_impl_trait)]
#![feature(return_position_impl_trait_in_trait)]
#![feature(async_fn_in_trait)]
pub mod core {
    // use async_trait::async_trait;
    use std::future::Future;
    // use std::sync::{Arc, Mutex};
    use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
    use tokio::runtime;
    use tokio::sync::oneshot;
    use tokio::task;
    use wgpu::{util::DeviceExt, Surface};
    use winit::{
        event::*,
        event_loop::{ControlFlow, EventLoop},
        window::{Window, WindowBuilder},
    };
    pub trait HasCreateInfo {
        type CreateInfo;
        fn new<'info>(
            create_info: &'info Self::CreateInfo,
        ) -> impl Future<Output = Self> + Send + 'info;
    }
    pub trait HasGenericCreateInfo<CreateInfo> {
        fn new<'info>(create_info: &'info CreateInfo) -> impl Future<Output = Self> + Send + 'info;
    }
    pub trait WindowAbstractor {
        fn get_size(&self) -> [u32; 2];
        fn resize(&mut self, new_size: [u32; 2]);
    }
    pub trait RustWindowAbstractor: WindowAbstractor {
        type Window: HasRawWindowHandle + HasRawDisplayHandle;
        fn get_window(&self) -> &Window;
    }
    pub trait GPUAbstractor
    where
        Self: Send + 'static,
    {
        fn resize(&mut self, new_size: [u32; 2]);
    }
    pub trait RendererAbstractor<WindowInterface, GPUInterface>
    where
        WindowInterface: WindowAbstractor,
        GPUInterface: GPUAbstractor,
    {
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
    struct WGPUInterface {
        surface: wgpu::Surface,
        device: wgpu::Device,
        queue: wgpu::Queue,
        config: wgpu::SurfaceConfiguration,
        size: winit::dpi::PhysicalSize<u32>,
    }
    struct WGPUInterfaceCreateInfo<'info, W>
    where
        W: RustWindowAbstractor + Send,
    {
        instance_descriptor: wgpu::InstanceDescriptor,
        device_descriptor: wgpu::DeviceDescriptor<'info>,
        window: &'info W,
    }
    impl<'info, W> WGPUInterfaceCreateInfo<'info, W>
    where
        W: RustWindowAbstractor + Send,
    {
        fn new(window: &'info W) -> Self {
            Self {
                instance_descriptor: wgpu::InstanceDescriptor {
                    backends: wgpu::Backends::PRIMARY,
                    ..Default::default()
                },
                device_descriptor: wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                window,
            }
        }
    }
    impl<W> HasGenericCreateInfo<WGPUInterfaceCreateInfo<'_, W>> for WGPUInterface
    where
        W: RustWindowAbstractor + Send + Sync,
    {
        fn new<'info>(
            create_info: &'info WGPUInterfaceCreateInfo<W>,
        ) -> impl Future<Output = Self> + Send + 'info {
            async {
                let size = {
                    let size = create_info.window.get_size();
                    winit::dpi::PhysicalSize {
                        width: size[0],
                        height: size[1],
                    }
                };
                let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                    backends: wgpu::Backends::all(),
                    ..Default::default()
                });

                let surface =
                    unsafe { instance.create_surface(&create_info.window.get_window()) }.unwrap();
                let adapter = instance
                    .request_adapter(&wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::default(),
                        compatible_surface: Some(&surface),
                        force_fallback_adapter: false,
                    })
                    .await
                    .unwrap();

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
                    view_formats: surface.get_capabilities(&adapter).formats,
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
    // impl GPUAbstractor for WGPUInterface {
    // fn resize(&mut self, new_size: Option<winit::dpi::PhysicalSize<u32>>) {
    //     let real_size = new_size.unwrap_or(self.size);
    //     if real_size.width > 0 && real_size.height > 0 {
    //         self.size = real_size;
    //         self.config.width = real_size.width;
    //         self.config.height = real_size.height;
    //         self.surface.configure(&self.device, &self.config);
    //         // self.renderer.resize(new_size);
    //     }
    // }
    // fn get_size(&self) -> winit::dpi::PhysicalSize<u32> {
    //     self.size.clone()
    // }
    // fn get_window(&self) -> &Window {
    //     self.renderer.get_window()
    // }
    // }
}
