#![feature(type_alias_impl_trait)]
#![feature(return_position_impl_trait_in_trait)]
#![feature(async_fn_in_trait)]
mod core {
    mod engine {
        use async_trait::async_trait;
        use std::future::Future;
        use std::sync::{Arc, Mutex};
        use tokio::runtime;
        use tokio::sync::oneshot;
        use tokio::task;
        use wgpu::{util::DeviceExt, Surface};
        use winit::{
            event::*,
            event_loop::{ControlFlow, EventLoop},
            window::{Window, WindowBuilder},
        };

        pub trait Renderer: Send + Sync {
            // fn new(window: Window) -> impl Future<Output = Arc<Self>> + Send;
            async fn new(window: Window) -> Self;
            fn render(&mut self) -> Result<(), wgpu::SurfaceError>;
            fn resize(&mut self, new_size: Option<winit::dpi::PhysicalSize<u32>>);
            fn get_size(&self) -> winit::dpi::PhysicalSize<u32>;
            fn get_window(&self) -> &Window;
        }

        struct WGPUInterface<GenericRenderer>
        where
            GenericRenderer: Renderer,
        {
            surface: wgpu::Surface,
            device: wgpu::Device,
            queue: wgpu::Queue,
            config: wgpu::SurfaceConfiguration,
            size: winit::dpi::PhysicalSize<u32>,
            renderer: GenericRenderer,
            // renderer: Arc<GenericRenderer>, // window: Window,d
        }
        struct Instance {}
        struct RT<GenericRenderer>
        where
            GenericRenderer: Renderer + 'static,
        {
            runtime: tokio::runtime::Runtime,
            renderer: GenericRenderer,
            event_loop: EventLoop<()>,
            // instance: Instance,
        }
        pub struct RTCreateInfo<GenericRenderer>
        where
            GenericRenderer: Renderer + 'static,
        {
            runtime: tokio::runtime::Runtime,
            renderer: GenericRenderer,
            event_loop: EventLoop<()>,
        }

        impl<GenericRenderer> RT<GenericRenderer>
        where
            GenericRenderer: Renderer + 'static,
        {
            fn new(create_info: RTCreateInfo<GenericRenderer>) -> Self {
                let RTCreateInfo {
                    runtime,
                    renderer,
                    event_loop,
                } = create_info;
                Self {
                    runtime,
                    renderer,
                    event_loop,
                }
            }
            fn run(mut self) {
                let RT {
                    mut runtime,
                    mut renderer,
                    mut event_loop,
                } = self;

                // runtime.block_on(async move {
                    // runtime.spawn_blocking(move || {
                    let mut time = std::time::Instant::now();
                    let mut dt = std::time::Duration::ZERO;
                    event_loop.run(move |event, __, control_flow| {
                        dt = std::time::Instant::now() - time;
                        time += dt;
                        println!("{:?}", dt);
                        match event {
                                Event::WindowEvent {
                                    ref event,
                                    window_id,
                                } if window_id == renderer.get_window().id()
                                    // && !instance.input(event)
                                    =>
                                {
                                    match event {
                                        WindowEvent::CloseRequested
                                        | WindowEvent::KeyboardInput {
                                            input:
                                                KeyboardInput {
                                                    state: ElementState::Pressed,
                                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                                    ..
                                                },
                                            ..
                                        } => *control_flow = ControlFlow::Exit,
                                        WindowEvent::Resized(physical_size) => {
                                            renderer.resize(Some(*physical_size));
                                        }
                                        WindowEvent::ScaleFactorChanged {
                                            new_inner_size, ..
                                        } => {
                                           renderer.resize(Some(**new_inner_size));
                                        }
                                        _ => {}
                                    }
                                }
                                Event::RedrawRequested(window_id)
                                    if window_id == renderer.get_window().id() =>
                                {
                                    // runtime.spawn(async {
                                    match renderer.render() {
                                        Ok(_) => {}
                                        Err(wgpu::SurfaceError::Lost) => {
                                            renderer.resize(None)
                                        }
                                        Err(wgpu::SurfaceError::OutOfMemory) => {
                                            *control_flow = ControlFlow::Exit
                                        }
                                        Err(e) => eprintln!("{:?}", e),
                                    }
                                }
                                _ => {}
                            }
                    // });
                    // });
                });
            }
        }
        impl<GenericRenderer> Default for RTCreateInfo<GenericRenderer>
        where
            GenericRenderer: Renderer, // GenericRenderer: Renderer + 'static,
        {
            fn default() -> Self {
                #[cfg(feature = "full")]
                let runtime = tokio::runtime::Builder::new_multi_thread().build().unwrap();
                #[cfg(not(feature = "full"))]
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .build()
                    .unwrap();

                let result = runtime.block_on(async move{
                    cfg_if::cfg_if! {
                        if #[cfg(target_arch = "wasm32")]{
                            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
                            console_log::init_with_level(log::Level::Warn).expect("couldnt initialize logger");
                        }else{
                            env_logger::init();
                        }
                    }
                    let event_loop = EventLoop::new();
                    let window = WindowBuilder::new().build(&event_loop).unwrap();
                    #[cfg(target_arch = "wasm32")]
                    {
                        use winit::dpi::PhysicalSize;
                        window.set_inner_size(PhysicalSize::new(450, 400));

                        use winit::platform::web::WindowExtWebSys;
                        web_sys::window()
                            .and_then(|win| win.document())
                            .and_then(|doc| {
                                let dst = doc.get_element_by_id("wasm-example")?;
                                let canvas = web_sys::Element::from(window.canvas());
                                dst.append_child(&canvas).ok()?;
                                Some(())
                            })
                            .expect("Couldn't append canvas to document body.");
                    }
                    let renderer = task::spawn_blocking(||{GenericRenderer::new(window)});
                    
                    // let instance = Instance {};
                    // let renderer = renderer.await.unwrap();
                    // let renderer = GenericRenderer::new(window.clone()).await;
                    (renderer.await,event_loop)
                });
                RTCreateInfo {
                    runtime,
                    renderer: result.0,
                    event_loop: result.1,
                }
            }
        }

        impl<GenericRenderer> Renderer for WGPUInterface<GenericRenderer>
        where
            GenericRenderer: Renderer,
        {
            async fn new(window: Window) -> Self {
                let size = window.inner_size();

                let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                    backends: wgpu::Backends::all(),
                    ..Default::default()
                });

                let surface = unsafe { instance.create_surface(&window) }.unwrap();
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
                let renderer = GenericRenderer::new(window).await;
                Self {
                    surface,
                    device,
                    queue,
                    config,
                    size,
                    renderer,
                }
            }
            fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
                self.renderer.render()
            }

            fn resize(&mut self, new_size: Option<winit::dpi::PhysicalSize<u32>>) {
                let real_size = new_size.unwrap_or(self.size);
                if real_size.width > 0 && real_size.height > 0 {
                    self.size = real_size;
                    self.config.width = real_size.width;
                    self.config.height = real_size.height;
                    self.surface.configure(&self.device, &self.config);
                    self.renderer.resize(new_size);
                }
            }
            fn get_size(&self) -> winit::dpi::PhysicalSize<u32> {
                self.size.clone()
            }
            fn get_window(&self) -> &Window {
                self.renderer.get_window()
            }
        }
        impl Instance {
            fn input(&mut self, event: &WindowEvent) -> bool {
                false
            }
        }
    }
}
