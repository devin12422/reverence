struct Renderer {
    gpu: reverence::WGPUInterface,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
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
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use tokio::sync::watch;
use tokio::sync::watch::*;
impl Renderer {
    fn render(&mut self) {
        match {
            let output = self.gpu.surface.get_current_texture().unwrap();
            let view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let mut encoder =
                self.gpu
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Render Encoder"),
                    });
            self.life.step(&mut command_encoder);
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                render_pass
                    .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
            }
            self.gpu.queue.submit(std::iter::once(encoder.finish()));
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
            }
        }
    }
}
