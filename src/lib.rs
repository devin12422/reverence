mod core {
    mod engine {
        use async_trait::async_trait;
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
        #[async_trait]
        trait Renderer {
            // Self and GenericRenderer should be interchangable
            async fn new(window: &Window) -> Self;
            fn init(&mut self);
            async fn render(&mut self) -> Result<(), wgpu::SurfaceError>;
            async fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>);
            fn size(&self) -> &winit::dpi::PhysicalSize<u32>;
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
            renderer: GenericRenderer, // window: Window,
        }
        struct Instance {}
        struct RT<GenericRenderer>
        where
            GenericRenderer: Renderer,
        {
            runtime: tokio::runtime::Runtime,
            renderer: GenericRenderer,
            event_loop: EventLoop<()>,
            window: Window,
            instance: Instance,
        }
        impl<GenericRenderer> RT<GenericRenderer>
        where
            GenericRenderer: Renderer + 'static,
        {
            async fn new() -> Self {
                cfg_if::cfg_if! {
                    if #[cfg(target_arch = "wasm32")]{
                        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
                        console_log::init_with_level(log::Level::Warn).expect("couldnt initialize logger");
                    }else{
                        env_logger::init();
                    }
                }
                #[cfg(feature = "full")]
                let runtime = tokio::runtime::Builder::new_multi_thread().build().unwrap();
                #[cfg(not(feature = "full"))]
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .build()
                    .unwrap();
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
                let renderer = GenericRenderer::new(&window).await;
                let instance = Instance {};
                Self {
                    runtime,
                    renderer,
                    event_loop,
                    window,
                    instance,
                }
            }
            fn run(mut self) {
                let RT {
                    mut runtime,
                    mut renderer,
                    mut event_loop,
                    mut window,
                    mut instance,
                } = self;
                runtime.block_on(async move {
                    event_loop.run(move |event, __, control_flow| {
                        // runtime.spawn(async move {
                        match event {
                            Event::WindowEvent {
                                ref event,
                                window_id,
                            } if window_id == window.id() && !instance.input(event) => {
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
                                        renderer.resize(*physical_size);
                                    }
                                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                                        renderer.resize(**new_inner_size);
                                    }
                                    _ => {}
                                }
                            }
                            Event::RedrawRequested(window_id) if window_id == window.id() => {
                                runtime.spawn(async move {
                                    // let result = wgpu.render().await;
                                    match wgpu.render().await {
                                        Ok(_) => {}
                                        Err(wgpu::SurfaceError::Lost) => {
                                            wgpu.resize(wgpu.size).await
                                        }
                                        Err(wgpu::SurfaceError::OutOfMemory) => {
                                            *control_flow = ControlFlow::Exit
                                        }
                                        Err(e) => eprintln!("{:?}", e),
                                    }
                                });
                            }
                            _ => {}
                        }
                        // });
                    });
                });
            }
        }
        #[async_trait::async_trait]
        impl<GenericRenderer> Renderer for WGPUInterface<GenericRenderer>
        where
            GenericRenderer: Renderer,
        {
            async fn new(window: &Window) -> Self {
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
            fn init(&mut self) {
                // <GenericRenderer as Renderer>::init(self.renderer);
                GenericRenderer::init(&mut self.renderer);
            }
            async fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
                self.renderer.render().await
            }
            async fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
                if new_size.width > 0 && new_size.height > 0 {
                    self.size = new_size;
                    self.config.width = new_size.width;
                    self.config.height = new_size.height;
                    self.surface.configure(&self.device, &self.config);
                    GenericRenderer::resize(self).await;
                }
            }
            fn size(&self) -> &winit::dpi::PhysicalSize<u32> {
                self.size
            }
        }
        impl Instance {
            fn input(&mut self, event: &WindowEvent) -> bool {
                false
            }
        }
    }
}
