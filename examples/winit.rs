#![feature(type_alias_impl_trait)]
#![feature(return_position_impl_trait_in_trait)]
#![feature(async_fn_in_trait)]
extern crate reverence;
// mod core;
// use crate::reverence::;
use reverence::*;
// use reverence;
// use reverence::core;
// use reverence::WGPUInterface;
// use core::WGPUInterface;
use bytemuck::{Pod, Zeroable};
use futures::future::BoxFuture;
// use reverence::core::*;
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
trait System {
    // type R
    fn should_call() -> bool;
    fn call();
}
#[derive(Debug)]
enum RendererInput {
    Render,
    Resize([u32; 2]),
    Exit,
}
use std::pin::Pin;
// #[repr(C)]
// #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
// struct Vertex {
//     position: [f32; 3],
//     color: [f32; 3],
// }
struct WindowHandler
where
    Self: 'static,
{
    window: Arc<Window>,
    event_loop: EventLoop<()>,
}
// use tokio::sync::Notify;
// struct Renderer {
//     gpu: reverence::WGPUInterface,
//     render_pipeline: wgpu::RenderPipeline,
//     vertex_buffer: wgpu::Buffer,
//     num_vertices: u32,
//     index_buffer: wgpu::Buffer,
//     num_indices: u32,
// }
impl WindowAbstractor for WindowHandler {
    fn get_size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.window.inner_size()
    }
}
impl WindowHandler {
    fn get_window(&self) -> Arc<winit::window::Window> {
        self.window.clone()
    }
}

// impl Vertex {
//     const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0=>Float32x3,1=>Float32x3];
//     fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
//         use std::mem;
//         wgpu::VertexBufferLayout {
//             array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
//             step_mode: wgpu::VertexStepMode::Vertex,
//             attributes: &Self::ATTRIBS,
//         }
//     }
// }
use tokio::sync::watch;
use tokio::sync::watch::*;
// impl Renderer {
// type SystemicFuture = impl Future<Output = Pin<Box<Self::SystemicFuture>>> + Send + 'static;
// fn run(self) -> Self::SystemicFuture;
// type SystemicFuture = impl Future<Output = Self::SystemicFuture> + Send + 'static;
// fn renderfn() ->
// }
// use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
// impl Renderer {
//     fn render(&mut self) {
//         //impl Future<Output = ()> + Send + 'static{
//         match {
//             let output = self.gpu.surface.get_current_texture().unwrap();
//             let view = output
//                 .texture
//                 .create_view(&wgpu::TextureViewDescriptor::default());
//             let mut encoder =
//                 self.gpu
//                     .device
//                     .create_command_encoder(&wgpu::CommandEncoderDescriptor {
//                         label: Some("Render Encoder"),
//                     });
//             {
//                 let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
//                     label: Some("Render Pass"),
//                     color_attachments: &[Some(wgpu::RenderPassColorAttachment {
//                         view: &view,
//                         resolve_target: None,
//                         ops: wgpu::Operations {
//                             load: wgpu::LoadOp::Clear(wgpu::Color {
//                                 r: 0.1,
//                                 g: 0.2,
//                                 b: 0.3,
//                                 a: 1.0,
//                             }),
//                             store: true,
//                         },
//                     })],
//                     depth_stencil_attachment: None,
//                 });
//                 render_pass.set_pipeline(&self.render_pipeline);
//                 render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
//                 render_pass
//                     .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
//                 render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
//             }
//             self.gpu.queue.submit(std::iter::once(encoder.finish()));
//             output.present();
//             Ok(())
//         } {
//             Ok(_) => {}
//             Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
//                 self.gpu.resize(self.gpu.size)
//             }
//             Err(wgpu::SurfaceError::OutOfMemory) => {
//                 println!("render broke");
//             }
//             Err(e) => eprintln!("{:?}", e),
//         };
//     }
//     fn new<W>(
//         window: Arc<W>,
//         size: impl Into<[u32; 2]>,
//     ) -> impl Future<Output = Self> + Send + 'static
//     where
//         W: HasRawWindowHandle + HasRawDisplayHandle + 'static,
//     {
//         let size = size.into();
//         let gpu = WGPUInterface::new(window, size);
//         async move {
//             let gpu = gpu.await;
//             // let renderer = task::spawn(GenericRenderer::new(window));
//             let shader = gpu
//                 .device
//                 .create_shader_module(wgpu::ShaderModuleDescriptor {
//                     label: Some("Shader"),
//                     source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
//                 }); // let instance = Instance {};
//                     // let renderer = renderer.await.unwrap();
//             let render_pipeline_layout =
//                 gpu.device
//                     .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
//                         label: Some("Render Pipeline Layout"),
//                         bind_group_layouts: &[],
//                         push_constant_ranges: &[],
//                     });
//             let render_pipeline =
//                 gpu.device
//                     .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
//                         label: Some("Render Pipeline"),
//                         layout: Some(&render_pipeline_layout),
//                         vertex: wgpu::VertexState {
//                             module: &shader,
//                             entry_point: "vs_main",
//                             buffers: &[Vertex::desc()],
//                         },
//                         fragment: Some(wgpu::FragmentState {
//                             module: &shader,
//                             entry_point: "fs_main",
//                             targets: &[Some(wgpu::ColorTargetState {
//                                 format: gpu.config.format,
//                                 blend: Some(wgpu::BlendState::REPLACE),
//                                 write_mask: wgpu::ColorWrites::ALL,
//                             })],
//                         }),
//                         primitive: wgpu::PrimitiveState {
//                             topology: wgpu::PrimitiveTopology::TriangleList,
//                             strip_index_format: None,
//                             front_face: wgpu::FrontFace::Ccw,
//                             cull_mode: Some(wgpu::Face::Back),
//                             polygon_mode: wgpu::PolygonMode::Fill,
//                             unclipped_depth: false,
//                             conservative: false,
//                         },
//                         depth_stencil: None,
//                         multisample: wgpu::MultisampleState {
//                             count: 1,
//                             mask: !0,
//                             alpha_to_coverage_enabled: false,
//                         },
//                         multiview: None,
//                     });
//             let vertex_buffer = gpu
//                 .device
//                 .create_buffer_init(&wgpu::util::BufferInitDescriptor {
//                     label: Some("Vertex Buffer"),
//                     contents: bytemuck::cast_slice(VERTICES),
//                     usage: wgpu::BufferUsages::VERTEX,
//                 });
//             let index_buffer = gpu
//                 .device
//                 .create_buffer_init(&wgpu::util::BufferInitDescriptor {
//                     label: Some("Index Buffer"),
//                     contents: bytemuck::cast_slice(INDICIES),
//                     usage: wgpu::BufferUsages::INDEX,
//                 });

//             let num_vertices = VERTICES.len() as u32;
//             let num_indices = INDICIES.len() as u32;
//             Self {
//                 gpu,
//                 render_pipeline,
//                 vertex_buffer,
//                 index_buffer,
//                 num_indices,
//                 num_vertices,
//             }
//         }
//     }
// }

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
// use instant::{Duration, Instant};
// #[cfg(target_arch = "wasm32")]
// use wasm_bindgen::prelude::wasm_bindgen;
// #[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
// fn main() {
//     #[cfg(feature = "full")]
//     let tokio_runtime = Arc::new(tokio::runtime::Builder::new_multi_thread().build().unwrap());
//     #[cfg(not(feature = "full"))]
//     let tokio_runtime = Arc::new(
//         tokio::runtime::Builder::new_current_thread()
//             .build()
//             .unwrap(),
//     );
//     let _guard = tokio_runtime.enter();
//     cfg_if::cfg_if! {
//         if #[cfg(target_arch = "wasm32")]{
//             std::panic::set_hook(Box::new(console_error_panic_hook::hook));
//             console_log::init_with_level(log::Level::Warn).expect("couldnt initialize logger");
//         }else{
//             env_logger::init();
//         }
//     }
//     let event_loop = EventLoop::new();
//     let window = WindowBuilder::new().build(&event_loop).unwrap();
//     #[cfg(target_arch = "wasm32")]
//     {
//         use winit::dpi::PhysicalSize;
//         window.set_inner_size(PhysicalSize::new(450, 400));

//         use winit::platform::web::WindowExtWebSys;
//         web_sys::window()
//             .and_then(|win| win.document())
//             .and_then(|doc| {
//                 let dst = doc.get_element_by_id("canvas")?;
//                 let canvas = web_sys::Element::from(window.canvas());
//                 dst.append_child(&canvas).ok()?;
//                 Some(())
//             })
//             .expect("Couldn't append canvas to document body.");
//     }
//     let window_handler = WindowHandler {
//         window: Arc::new(window),
//         event_loop,
//     };
//     let (tx, mut rx) = watch::channel(RendererInput::Render);
//     let w = window_handler.get_window();
//     let mut renderer = tokio_runtime.block_on(Renderer::new(
//         w,
//         *(&window_handler.get_size())));
//     let mut time = Instant::now();
//     let mut dt = Duration::ZERO;
//     let WindowHandler { window, event_loop } = window_handler;
//     let notify = Arc::new(Notify::new());
//     let notify2 = notify.clone();
//     tokio_runtime.block_on(async{
//         tokio_runtime.spawn(async move{
//             loop{
//                 if rx.has_changed().unwrap(){
//                     match *rx.borrow_and_update(){
//                         RendererInput::Render =>{
//                             renderer.render();
//                         },
//                         RendererInput::Resize(size)=>{
//                             renderer.gpu.resize(size);
//                         },
//                         RendererInput::Exit=>{
//                             break;
//                         }
//                     };
//                 }
//                 notify.notify_one();
//                 task::yield_now().await;
//             }
//             notify.notify_one();
//         });
//    });
//    let mut last_frame = Duration::ZERO;
//    event_loop.run(move |event, _, control_flow| {
//        dt = Instant::now() - time;
//        time = time + dt;
//        last_frame = last_frame + dt;
//        match event{
//            Event::WindowEvent {
//                ref event,
//                window_id,
//            } if window_id == window.id()
//                // && !instance.input(event)
//                =>
//            {
//                match event {
//                    WindowEvent::CloseRequested
//                    | WindowEvent::KeyboardInput {
//                        input:
//                            KeyboardInput {
//                                state: ElementState::Pressed,
//                                virtual_keycode: Some(VirtualKeyCode::Escape),
//                                ..
//                            },
//                        ..
//                    } => {
//                        tx.send(RendererInput::Exit).unwrap();
//                        *control_flow = ControlFlow::Exit
//                    },
//                    WindowEvent::Resized(physical_size) => {
//                        tx.send(RendererInput::Resize((*physical_size).into())).unwrap();
//                    }
//                    WindowEvent::ScaleFactorChanged {
//                        new_inner_size, ..
//                    } => {
//                        tx.send(RendererInput::Resize((**new_inner_size).into())).unwrap();
//                    }
//                    _ => {}
//                }
//            }
//            Event::RedrawRequested(window_id)
//                if window_id == window.id() =>
//            {
//                tx.send(RendererInput::Render).unwrap();
//                tokio_runtime.block_on(notify2.notified());
//                last_frame = Duration::ZERO;
//             },Event::MainEventsCleared =>{

//                if last_frame.as_millis() >= 16{
//                    window.request_redraw();
//                }
//         }
//            _ => {}
//        }
//    });
// }
