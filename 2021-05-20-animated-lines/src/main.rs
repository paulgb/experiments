use std::iter;

use std::time::{Instant};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BlendComponent, BlendState, BufferUsage};
use winit::dpi::PhysicalSize;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use rand::{thread_rng, Rng, seq::SliceRandom};

const NUM_LINES: u32 = 30000;

const OPTIONS: [f32; 11] = [6., 8., 10., 12., 15., 20., 24., 30., 40., 60., 120.];
const OFFSET: f32 = std::f32::consts::PI / 3.;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable, bytemuck::Pod)]
struct Line {
    start: [f32; 2],
    end: [f32; 2],
    color: [f32; 4],
    width: f32,
}

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,

    instance_buffer: wgpu::Buffer,
    created: Instant,
    generators: Vec<LineGenerator>,
}

struct LineGenerator {
    start: [f32; 2],
    speed: f32,
    color: [f32; 4],
    width: f32,
    length: f32,
}

impl LineGenerator {
    pub fn new_rand() -> Self {
        let mut rng = thread_rng();

        LineGenerator {
            start: [rng.gen_range(-1.1..1.1), rng.gen_range(-1.1..1.1)],
            speed: *OPTIONS.choose(&mut rng).unwrap(),
            color: [
                rng.gen_range(0.0..1.0),
                rng.gen_range(0.0..1.0),
                rng.gen_range(0.0..1.0),
                1.0
            ],
            width: rng.gen_range(0.001..0.005),
            length: rng.gen_range(0.01..0.3),
        }
    }

    pub fn gen_line(&self, time: f32) -> Line {
        let rot = OFFSET + 4. * time / self.speed;

        let end = [
            self.start[0] + self.length * rot.cos(),
            self.start[1] + self.length * rot.sin(),
        ];

        Line {
            start: self.start,
            end,
            color: self.color,
            width: self.width
        }
    }
}

impl State {
    fn gen_lines(generators: &[LineGenerator], time: f32) -> Vec<Line> {
        generators.iter().map(|d| d.gen_line(time)).collect()
    }

    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let generators: Vec<LineGenerator> = (0..NUM_LINES).map(|_| LineGenerator::new_rand()).collect();

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance buffer"),
            contents: bytemuck::cast_slice(&Self::gen_lines(&generators, 0.)),
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
        });

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let vs_module = device.create_shader_module(&wgpu::include_spirv!("shader.vert.spv"));
        let fs_module = device.create_shader_module(&wgpu::include_spirv!("shader.frag.spv"));

        let instance_buffer_desc = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Line>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        };

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[instance_buffer_desc],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: sc_desc.format,
                    write_mask: wgpu::ColorWrite::ALL,
                    blend: Some(BlendState {
                        color: BlendComponent::OVER,
                        alpha: BlendComponent::REPLACE,
                    }),
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                clamp_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });

        let created = Instant::now();

        Self {
            surface,
            device,
            queue,
            size,
            sc_desc,
            swap_chain,
            render_pipeline,
            instance_buffer,
            created,
            generators,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        let frame = self.swap_chain.get_current_frame()?.output;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let time = self.created.elapsed().as_secs_f32();

        let lines = Self::gen_lines(&self.generators, time);
        let tmp_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Temporary buffer"),
            contents: &bytemuck::cast_slice(&lines),
            usage: BufferUsage::COPY_SRC,
        });

        encoder.copy_buffer_to_buffer(
            &tmp_buffer,
            0,
            &self.instance_buffer,
            0,
            std::mem::size_of::<Line>() as u64 * NUM_LINES as u64,
        );

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.,
                            g: 0.,
                            b: 0.,
                            a: 1.,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.instance_buffer.slice(..));
            render_pass.draw(0..6, 0..NUM_LINES);
        }

        self.queue.submit(iter::once(encoder.finish()));

        Ok(())
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(1300, 1300))
        .with_title("Alignment")
        .build(&event_loop)
        .unwrap();

    use futures::executor::block_on;

    let mut state = block_on(State::new(&window));

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::KeyboardInput { input, .. } => match input {
                KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Escape),
                    ..
                } => *control_flow = ControlFlow::Exit,
                _ => {}
            },
            WindowEvent::Resized(physical_size) => {
                state.resize(*physical_size);
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                state.resize(**new_inner_size);
            }
            _ => {}
        },
        Event::RedrawRequested(_) => match state.render() {
            Ok(_) => {}
            Err(wgpu::SwapChainError::Lost) => state.resize(state.size),
            Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
            Err(e) => eprintln!("{:?}", e),
        },
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        _ => {}
    });
}
