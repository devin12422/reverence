// extern crate muggs;
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
        trait Renderer<GenericRenderer>
        where
            GenericRenderer: Renderer<GenericRenderer>,
        {
            // Self and GenericRenderer should be interchangable
            async fn new() -> Self;
            async fn init(wgpu: &mut WGPUInterface<GenericRenderer>);
            async fn render(wgpu: &mut WGPUInterface<GenericRenderer>);
            async fn resize(wgpu: &mut WGPUInterface<GenericRenderer>);
        }
        struct WGPUInterface<GenericRenderer>
        where
            GenericRenderer: Renderer<GenericRenderer>,
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
            GenericRenderer: Renderer<GenericRenderer>,
        {
            runtime: tokio::runtime::Runtime,
            wgpu: WGPUInterface<GenericRenderer>,
            event_loop: EventLoop<()>,
            window: Window,
            instance: Instance,
        }
        impl<GenericRenderer> RT<GenericRenderer>
        where
            GenericRenderer: Renderer<GenericRenderer> + 'static,
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
                let wgpu = WGPUInterface::new(&window).await;
                let instance = Instance {};
                Self {
                    runtime,
                    wgpu,
                    event_loop,
                    window,
                    instance,
                }
            }
            fn run(mut self) {
                let RT {
                    mut runtime,
                    mut wgpu,
                    mut event_loop,
                    mut window,
                    mut instance,
                } = self;
                runtime.block_on(async move {
                    event_loop.run(move |event, __, control_flow| {
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
                                        wgpu.resize(*physical_size);
                                    }
                                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                                        wgpu.resize(**new_inner_size);
                                    }
                                    _ => {}
                                }
                            }
                            Event::RedrawRequested(window_id) if window_id == window.id() => {
                                runtime.spawn_blocking(|| async { wgpu.render() });
                                // wgpu.render();
                                // match state.render() {
                                //     Ok(_) => {}
                                //     Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                                //     Err(wgpu::SurfaceError::OutOfMemory) => {
                                //         *control_flow = ControlFlow::Exit
                                //     }
                                //     Err(e) => eprintln!("{:?}", e),
                                // }
                            }
                            _ => {}
                        }
                    });
                });
            }
        }
        impl<GenericRenderer> WGPUInterface<GenericRenderer>
        where
            GenericRenderer: Renderer<GenericRenderer>,
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
                let renderer = GenericRenderer::new().await;
                Self {
                    surface,
                    device,
                    queue,
                    config,
                    size,
                    renderer,
                }
            }
            async fn init(&mut self) {
                // <GenericRenderer as Renderer>::init(self.renderer);
                GenericRenderer::init(self).await;
            }
            async fn render(&mut self) {
                GenericRenderer::render(self).await;
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
        }
        impl Instance {
            fn input(&mut self, event: &WindowEvent) -> bool {
                false
            }
        }
    }
}
