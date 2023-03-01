// extern crate muggs;
mod core {
    mod engine {
        use std::sync::{Arc, Mutex};
        use tokio::runtime::Runtime;
        use tokio::sync::oneshot;
        use wgpu::{util::DeviceExt, Surface};
        use winit::{
            event::*,
            event_loop::{ControlFlow, EventLoop},
            window::{Window, WindowBuilder},
        };

        struct WGPUInterface {
            surface: wgpu::Surface,
            device: wgpu::Device,
            queue: wgpu::Queue,
            config: wgpu::SurfaceConfiguration,
            size: winit::dpi::PhysicalSize<u32>,
            window: Window,
        }
        struct Renderer {}
        struct RT {
            runtime: tokio::runtime::Runtime,
            wgpu: WGPUInterface,
            event_loop: EventLoop<()>,
            window: Window,
        }
        impl RT {
            fn new() -> Self {
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
              let size = window.inner_size();

                let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                    backends: wgpu::Backends::all(),
                    ..Default::default()
                });

                let surface = unsafe { instance.create_surface(&window) }.unwrap();
                let (tx,rx) = oneshot::channel();
                runtime.spawn( async move { let adapter = instance
                   .request_adapter(&wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::default(),
                        compatible_surface: Some(&surface),
                        force_fallback_adapter: false,
                    }).await;
                tx.send(adapter).unwrap();
                });
                runtime.block_on(async move {
                    
                let adapter = rx.await.unwrap();
                });
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
                let wgpu = WGPUInterface{surface,device,queue,config,size}                Self { runtime, wgpu }
            }
            fn run() {}
        }
        impl WGPUInterface {
            fn ew(runtime: tokio::runtime::Runtime) {}
            async fn new(surface:Surface) -> Self {
                
                Self {
                                        surface,
                    device,
                    queue,
                    config,
                    size,
                }
            }
            pub fn window(&self) -> &Window {
                &self.window
            }

            fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
                if new_size.width > 0 && new_size.height > 0 {
                    self.size = new_size;
                    self.config.width = new_size.width;
                    self.config.height = new_size.height;
                    self.surface.configure(&self.device, &self.config);
                }
            }
            fn input(&mut self, event: &WindowEvent) -> bool {
                false
            }
            fn update(&mut self) {}
        }

        pub fn run() {
            let mut state = pollster::block_on(WGPUInterface::new(window));
            event_loop.run(move |event, __, control_flow| match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == state.window().id() && !state.input(event) => match event {
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
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                },
                Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                    state.update();
                    // match state.render() {
                    //     Ok(_) => {}
                    //     Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    //     Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    //     Err(e) => eprintln!("{:?}", e),
                    // }
                }
                _ => {}
            });
        }
    }
}
// const VERTICES: &[Vertex] = &[
//     Vertex {
//         position: [-0.0868241, 0.49240386, 0.0],
//         color: [0.5, 0.0, 0.5],
//     },
//     Vertex {
//         position: [-0.49513406, 0.06958647, 0.0],
//         color: [0.5, 0.0, 0.5],
//     },
//     Vertex {
//         position: [-0.21918549, -0.44939706, 0.0],
//         color: [0.5, 0.0, 0.5],
//     },
//     Vertex {
//         position: [0.35966998, -0.3473291, 0.0],
//         color: [0.5, 0.0, 0.5],
//     },
//     Vertex {
//         position: [0.44147372, 0.2347359, 0.0],
//         color: [0.5, 0.0, 0.5],
//     },
// ];
// const INDICIES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];
