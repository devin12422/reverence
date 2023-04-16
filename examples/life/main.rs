#![feature(type_alias_impl_trait)]
#![feature(return_position_impl_trait_in_trait)]
#![feature(async_fn_in_trait)]
extern crate reverence;
// mod core;
// use crate::reverence::;
use reverence::*;

use rand::{
    distributions::{Distribution, Uniform},
    SeedableRng,
};
// use wgpu::SwapChainDescriptor;

// mod debug_buffer;
// mod dimensions;
mod directions;
mod life;
mod life_params;
mod texture;

use crate::{life::Life, life_params::LifeParams, texture::Texture};
use wgpu::*;
// ---------------------------------------------------------------------------

/// LifeProg struct holds all of the state used by the program.
struct LifeProg {
    life: Life,
    renderer: Renderer,
}

// use reverence;
// use reverence::core;
// use reverence::WGPUInterface;
// use core::WGPUInterface;
use bytemuck::{Pod, Zeroable};
use futures::future::BoxFuture;
use std::borrow::Cow;
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
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    _pos: [f32; 4],
    _tex_coord: [f32; 2],
}
struct WindowHandler
where
    Self: 'static,
{
    window: Arc<Window>,
    event_loop: EventLoop<()>,
}
use tokio::sync::Notify;
const SQUARE_POINTS: [u32; 8] = [0, 0, 0, 1, 1, 0, 1, 1];
struct Renderer {
    gpu: reverence::WGPUInterface,
    render_pipeline: wgpu::RenderPipeline,
    square_buffer: Buffer,
    // square_indices: Buffer,
    life: Life,
    bind_group: BindGroup,
}
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
const SQUARE_STRIDE: VertexBufferLayout = VertexBufferLayout {
    array_stride: 2 * std::mem::size_of::<u32>() as wgpu::BufferAddress,
    step_mode: VertexStepMode::Vertex,
    attributes: &wgpu::vertex_attr_array![1=>Uint32x2],
};

const CELL_STRIDE: VertexBufferLayout = VertexBufferLayout {
    array_stride: std::mem::size_of::<u32>() as wgpu::BufferAddress,
    step_mode: VertexStepMode::Instance,
    attributes: &wgpu::vertex_attr_array![0=>Uint32],
};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use tokio::sync::watch;
use tokio::sync::watch::*;
impl Renderer {
    fn render(&mut self) {
        match {
            let texture_result = self.gpu.surface.get_current_texture();
            if texture_result.is_err() {
                println!("{}", texture_result.err().unwrap());
                return;
            }
            let output = texture_result.unwrap();
            let view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            {
                let mut encoder =
                    self.gpu
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Render Encoder"),
                        });
                {
                    self.life.step(&mut encoder);

                    let mut render_pass =
                        &mut encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

                    render_pass.set_bind_group(0, &self.bind_group, &[]);
                    render_pass.set_vertex_buffer(0, self.life.src_buf().slice(..));

                    render_pass.set_vertex_buffer(1, self.square_buffer.slice(..));
                    render_pass.draw(0..4, 0..self.life.n_cells());
                }

                self.gpu.queue.submit(std::iter::once(encoder.finish()));
            }
            output.present();
            Ok(())
        } {
            Ok(_) => {}
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.gpu.resize(self.gpu.size)
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                println!("render broke");
            }
            Err(e) => eprintln!("{:?}", e),
        };
    }
    fn new<W>(
        window: Arc<W>,
        size: impl Into<[u32; 2]>,
    ) -> impl Future<Output = Self> + Send + 'static
    where
        W: HasRawWindowHandle + HasRawDisplayHandle + 'static,
    {
        let size = size.into();
        let gpu = WGPUInterface::new(window, size);
        async move {
            let gpu = gpu.await;
            let shader = gpu
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: None,
                    source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("render.wgsl"))),
                });
            let params = LifeParams::new(&gpu.device, [100, 100], 0.70);

            let mut life = Life::new(&gpu.device, [100, 100], &params);

            // Set the initial state for all cells in the life grid.
            life.import(&gpu.device, &gpu.queue, {
                let mut cell_data: Vec<u32> = Vec::new();
                let mut rng = rand::rngs::StdRng::seed_from_u64(42);
                let unif = Uniform::new_inclusive(0, 1);
                for _ in 0..life.n_cells() {
                    cell_data.push(unif.sample(&mut rng) as u32);
                }
                cell_data
            });
            let bind_group_layout =
                gpu.device
                    .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        label: None,
                        entries: &[wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: params.binding_type(),
                            count: None,
                        }],
                    });
            let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params.binding_resource(),
                }],
                label: None,
            });
            let render_pipeline_layout =
                gpu.device
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("Render Pipeline Layout"),
                        bind_group_layouts: &[&bind_group_layout],
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
                            buffers: &[CELL_STRIDE, SQUARE_STRIDE],
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
                            topology: wgpu::PrimitiveTopology::TriangleStrip,
                            // strip_index_format: None,
                            // front_face: wgpu::FrontFace::Ccw,
                            cull_mode: None, //Some(wgpu::Face::Front),
                            // polygon_mode: wgpu::PolygonMode::Fill,
                            // unclipped_depth: false,
                            // conservative: false,
                            ..Default::default()
                        },
                        depth_stencil: None,
                        multisample: wgpu::MultisampleState {
                            count: 1,
                            mask: !0,
                            alpha_to_coverage_enabled: false,
                        },
                        multiview: None,
                    });
            // let (vertex_data, index_data) = Renderer::create_vertices();
            let square_buffer = gpu
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&SQUARE_POINTS),
                    usage: wgpu::BufferUsages::VERTEX,
                });
            // let index_buffer = gpu
            //     .device
            //     .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            //         label: Some("Index Buffer"),
            //         contents: bytemuck::cast_slice(&index_data),
            //         usage: wgpu::BufferUsages::INDEX,
            //     });

            // let num_vertices = vertex_data.len() as u32;
            // let num_indices = index_data.len() as u32;

            Self {
                gpu,
                render_pipeline,
                life,
                square_buffer,
                bind_group,
            }
        }
    }
}
use instant::{Duration, Instant};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
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
                let dst = doc.get_element_by_id("canvas")?;
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
    let (tx, mut rx) = watch::channel(RendererInput::Render);

    let size = window_handler.get_size();
    let mut renderer = tokio_runtime.block_on(Renderer::new(window_handler.get_window(), *(&size)));

    // Step the algorithm a few times, so the initial image looks Life-like.
    // let mut command_encoder = renderer
    //     .gpu
    //     .device
    //     .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    // for _ in 0..100 {
    //     renderer.life.step(&mut command_encoder);
    // }
    // renderer.gpu.queue.submit(Some(command_encoder.finish()));
    let mut time = Instant::now();
    let mut dt = Duration::ZERO;
    let WindowHandler { window, event_loop } = window_handler;
    let notify = Arc::new(Notify::new());
    let notify2 = notify.clone();
    tokio_runtime.block_on(async {
        tokio_runtime.spawn(async move {
            loop {
                if rx.has_changed().unwrap() {
                    match *rx.borrow_and_update() {
                        RendererInput::Render => {
                            renderer.render();
                        }
                        RendererInput::Resize(size) => {
                            renderer.gpu.resize(size);
                        }
                        RendererInput::Exit => {
                            break;
                        }
                    };
                }
                notify.notify_one();
                task::yield_now().await;
            }
            notify.notify_one();
        });
    });
    let mut last_frame = Duration::ZERO;
    event_loop.run(move |event, _, control_flow| {
        dt = Instant::now() - time;
        time = time + dt;
        last_frame = last_frame + dt;
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
                   } => {
                       tx.send(RendererInput::Exit).unwrap();
                       tokio_runtime.block_on(notify2.notified());
                       *control_flow = ControlFlow::Exit
                   },
                   WindowEvent::Resized(physical_size) => {
                       tx.send(RendererInput::Resize((*physical_size).into())).unwrap();
                       tokio_runtime.block_on(notify2.notified());
                   }
                   WindowEvent::ScaleFactorChanged {
                       new_inner_size, ..
                   } => {
                       tx.send(RendererInput::Resize((**new_inner_size).into())).unwrap();
                       tokio_runtime.block_on(notify2.notified());
                   }
                   _ => {}
               }
           }
           Event::RedrawRequested(window_id)
               if window_id == window.id() =>
           {
               tx.send(RendererInput::Render).unwrap();
               tokio_runtime.block_on(notify2.notified());
               last_frame = Duration::ZERO;
            },Event::MainEventsCleared =>{

               if last_frame.as_millis() >= 16{
                   window.request_redraw();
               }
        }
           _ => {}
       }
    });
}
