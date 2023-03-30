#![feature(type_alias_impl_trait)]
#![feature(return_position_impl_trait_in_trait)]
#![feature(async_fn_in_trait)]
extern crate reverence;
use bytemuck::{Pod, Zeroable};
use futures::future::BoxFuture;
use reverence::core::*;
use std::future::Future;
use std::sync::Arc;
use tokio::runtime;
use tokio::sync::oneshot;
use tokio::task;
use wgpu::{util::DeviceExt, Surface};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
#[derive(Debug)]
enum RendererInput {
    Render,
    Resize([u32; 2]),
}
use std::pin::Pin;

trait AsyncFnMut<T> {
    type Fut: Future<Output = Self::Output>;
    type Output;

    fn call(&mut self, arg: T) -> Self::Fut;
}

impl<F, Fut, T> AsyncFnMut<T> for F
where
    F: FnMut(T) -> Fut,
    Fut: Future,
{
    type Fut = Fut;
    type Output = Fut::Output;

    fn call(&mut self, arg: T) -> Fut {
        (self)(arg)
    }
}
// trait Systemic
//  where
//      Self: Send + 'static,
//  {
// /
//      type SystemicFuture:Future<Output = Pin<Box<Self::SystemicFuture>>> + Send + 'static;
//      fn run() -> Pin<Box< Self::SystemicFuture>>;
// impl for<'a> AsyncFnMut<&'a mut File, Output = std::io::Result<()>>,
//  }
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}
struct WindowHandler
where
    Self: 'static,
{
    window: Arc<Window>,
    event_loop: EventLoop<()>,
}
use tokio::sync::Notify;
struct Renderer
// where
// Self: Systemic,
{
    gpu: WGPUInterface,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    rx: Receiver<RendererInput>,
    notify: Arc<Notify>,
}
impl WindowAbstractor for WindowHandler {
    fn get_size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.window.inner_size()
    }
}
impl RustWindowAbstractor for WindowHandler {
    type Window = winit::window::Window;
    fn get_window(&self) -> &Self::Window {
        &self.window
    }
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0=>Float32x3,1=>Float32x3];

    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}
use tokio::sync::watch;
use tokio::sync::watch::*;
// impl Renderer {
    // type SystemicFuture = impl Future<Output = Pin<Box<Self::SystemicFuture>>> + Send + 'static;
    // fn run(self) -> Self::SystemicFuture;
    // type SystemicFuture = impl Future<Output = Self::SystemicFuture> + Send + 'static;
    // fn renderfn() -> 
// }
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
impl Renderer {
    fn render(mut self) -> impl Future<Output = ()> + Send + 'static{
        async move{
            println!("starting renderer");
            if (self.rx.has_changed().unwrap()) {
                println!("recieved command");
                match match *self.rx.borrow_and_update() {
                    RendererInput::Render => {
                        let output = self.gpu.surface.get_current_texture().unwrap();
                        let view = output
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());
                        let mut encoder = self.gpu.device.create_command_encoder(
                            &wgpu::CommandEncoderDescriptor {
                                label: Some("Render Encoder"),
                            },
                        );
                        {
                            let mut render_pass =
                                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                    label: Some("Render Pass"),
                                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                        view: &view,
                                        resolve_target: None,
                                        ops: wgpu::Operations {
                                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                                r: 0.1,
                                                g: 0.2,
                                                b: 0.3,
                                                a: 1.0,
                                            }),
                                            store: true,
                                        },
                                    })],
                                    depth_stencil_attachment: None,
                                });
                            render_pass.set_pipeline(&self.render_pipeline);
                            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                            render_pass.set_index_buffer(
                                self.index_buffer.slice(..),
                                wgpu::IndexFormat::Uint16,
                            );
                            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
                        }
                        self.gpu.queue.submit(std::iter::once(encoder.finish()));
                        output.present();
                        Ok(())
                    }
                    RendererInput::Resize(size) => {
                        self.gpu.resize(size);
                        Ok(())
                    }
                } {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        self.gpu.resize(self.gpu.size)
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        println!("render broke");
                    }
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            println!("done rendering");
            // BoxFuture
            }
    }
    fn new<W>(
        window: Arc<W>,
        size: impl Into<[u32; 2]>,
        rx: Receiver<RendererInput>,
        notify: Arc<Notify>,
    ) -> impl Future<Output = Self> + Send + 'static
    where
        W: HasRawWindowHandle + HasRawDisplayHandle + Send + Sync + 'static,
    {
        let size = size.into();
        async move {
            let gpu = WGPUInterface::new(window.clone(), size).await;
            // let renderer = task::spawn(GenericRenderer::new(window));

            let shader = gpu
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Shader"),
                    source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
                }); // let instance = Instance {};
                    // let renderer = renderer.await.unwrap();

            let render_pipeline_layout =
                gpu.device
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("Render Pipeline Layout"),
                        bind_group_layouts: &[],
                        push_constant_ranges: &[],
                    });

            let render_pipeline =
                gpu.device
                    .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some("Render Pipeline"),
                        layout: Some(&render_pipeline_layout),
                        vertex: wgpu::VertexState {
                            module: &shader,
                            entry_point: "vs_main",
                            buffers: &[Vertex::desc()],
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &shader,
                            entry_point: "fs_main",
                            targets: &[Some(wgpu::ColorTargetState {
                                format: gpu.config.format,
                                blend: Some(wgpu::BlendState::REPLACE),
                                write_mask: wgpu::ColorWrites::ALL,
                            })],
                        }),
                        primitive: wgpu::PrimitiveState {
                            topology: wgpu::PrimitiveTopology::TriangleList,
                            strip_index_format: None,
                            front_face: wgpu::FrontFace::Ccw,
                            cull_mode: Some(wgpu::Face::Back),
                            polygon_mode: wgpu::PolygonMode::Fill,
                            unclipped_depth: false,
                            conservative: false,
                        },
                        depth_stencil: None,
                        multisample: wgpu::MultisampleState {
                            count: 1,
                            mask: !0,
                            alpha_to_coverage_enabled: false,
                        },
                        multiview: None,
                    });
            let vertex_buffer = gpu
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(VERTICES),
                    usage: wgpu::BufferUsages::VERTEX,
                });
            let index_buffer = gpu
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(INDICIES),
                    usage: wgpu::BufferUsages::INDEX,
                });

            let num_vertices = VERTICES.len() as u32;
            let num_indices = INDICIES.len() as u32;
            Self {
                gpu,
                render_pipeline,
                vertex_buffer,
                index_buffer,
                num_indices,
                num_vertices,
                rx,
                notify, // window: window_handler,
            }
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        color: [0.5, 0.0, 0.5],
    },
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        color: [0.5, 0.0, 0.5],
    },
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        color: [0.5, 0.0, 0.5],
    },
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        color: [0.5, 0.0, 0.5],
    },
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        color: [0.5, 0.0, 0.5],
    },
];
const INDICIES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

// async fn render(
//     renderer: &'static mut Renderer,
//     mut system_function: impl for<'a> AsyncFnMut<&'a mut Renderer, Output = ()>,
// ) {
//     system_function.call(renderer).await;
// }
fn main() {
    #[cfg(feature = "full")]
    let tokio_runtime = Arc::new(tokio::runtime::Builder::new_multi_thread().build().unwrap());
    #[cfg(not(feature = "full"))]
    let tokio_runtime = Arc::new(
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap(),
    );
    let _guard = tokio_runtime.enter();
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
    let window_handler = WindowHandler {
        window: Arc::new(window),
        event_loop,
    };
    // tokio_runtime.block_on(async move {
    // let window_handler = WindowHandler{window,event_loop};
    // let window_handler = window_handler;
    let (tx, mut rx) = watch::channel(RendererInput::Render);
    let render_notify = Arc::new(Notify::new());
    // window_handler.window.as_ref
    let renderer_task = task::spawn(Renderer::new(
        window_handler.window.clone(),
        *(&window_handler.get_size()),
        rx,
        render_notify.clone(),
    ));
    let mut time = std::time::Instant::now();
    let mut dt = std::time::Duration::ZERO;
    let mut renderer = tokio_runtime.block_on(renderer_task).unwrap();
    tx.send(RendererInput::Render).unwrap();
    let WindowHandler { window, event_loop } = window_handler;
    let mut render_task = Box::pin(renderer.render());
    // tokio::pin!(render_task);
    event_loop.run(move |event, _, control_flow| {
        dt = std::time::Instant::now() - time;
        time += dt;
        // println!("{:?}", dt);
        match event{
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id()
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
                        tx.send(RendererInput::Resize((*physical_size).into())).unwrap();
                        // gpu.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged {
                        new_inner_size, ..
                    } => {
                        tx.send(RendererInput::Resize((**new_inner_size).into())).unwrap();
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(window_id)
                if window_id == window.id() =>
            {

                tx.send(RendererInput::Render).unwrap();
                // let x = renderfn();
                // let render_task = tokio_runtime.spawn((&mut render_task));
                
                // {
                // let render_task = tokio_runtime.block_on(render_task);
                // let render_task = tokio_runtime.spawn(renderer.run());
                tokio_runtime.block_on((render_task.as_mut()));
                // let renderer = tokio_runtime.block_on(render_task);
                // }
                // pollster::block_on(render_notify.notified());
                println!("finished rendering");


                 // let render = task::spawn(render(&gpu,&render_pipeline,&vertex_buffer ,&num_vertices ,&index_buffer,&num_indices));
                // pollster::block_on(render);
             },Event::MainEventsCleared =>{
                window.request_redraw();

            }
            _ => {}
        }
        // event_loop.run(test);
        // });
    });
    // });
    // runtime.block_on(async move {
    // runtime.spawn_blocking(move || {
    // task::spawn(async{
    // }
}
