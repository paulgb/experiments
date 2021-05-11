use futures::executor::{LocalPool, LocalSpawner};
use futures::task::SpawnExt;
use std::error::Error;
use wgpu::util::StagingBelt;
use wgpu::{Device, Queue, Surface, SwapChain};
use wgpu_glyph::{ab_glyph, GlyphBrush, GlyphBrushBuilder, Section, Text};
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoopWindowTarget, EventLoop};

const RENDER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

struct TextProgram {
    pub size: PhysicalSize<u32>,
    pub swap_chain: SwapChain,
    pub glyph_brush: GlyphBrush<()>,
    pub surface: Surface,
    pub device: Device,
    pub staging_belt: StagingBelt,
    pub local_pool: LocalPool,
    pub local_spawner: LocalSpawner,
    pub queue: Queue,
    pub event_loop: EventLoop<()>,
}

impl TextProgram {
    pub fn handle_event(
        &mut self,
        event: Event<()>,
        _: &EventLoopWindowTarget<()>,
        control_flow: &mut ControlFlow,
    ) {
        match event {
            // Close.
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,

            // Resize.
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                self.size = new_size;
                // TODO: this is repeated
                self.swap_chain = self.device.create_swap_chain(
                    &self.surface,
                    &wgpu::SwapChainDescriptor {
                        usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
                        format: RENDER_FORMAT,
                        width: self.size.width,
                        height: self.size.height,
                        present_mode: wgpu::PresentMode::Mailbox,
                    },
                )
            }

            // Redraw.
            Event::RedrawRequested { .. } => {
                // Get a command encoder for the current frame
                let mut encoder =
                    self.device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Redraw"),
                        });

                // Get the next frame
                let frame = self
                    .swap_chain
                    .get_current_frame()
                    .expect("Get next frame")
                    .output;

                // Clear frame
                {
                    let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render pass"),
                        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &frame.view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.4,
                                    g: 0.4,
                                    b: 0.4,
                                    a: 1.0,
                                }),
                                store: true,
                            },
                        }],
                        depth_stencil_attachment: None,
                    });
                }

                self.glyph_brush.queue(Section {
                    screen_position: (30., 30.),
                    bounds: (self.size.width as f32, self.size.height as f32),
                    text: vec![Text::new("Hello, world!")
                        .with_color([0.0, 0.0, 0.0, 1.0])
                        .with_scale(40.0)],
                    ..Section::default()
                });

                self.glyph_brush
                    .draw_queued(
                        &self.device,
                        &mut self.staging_belt,
                        &mut encoder,
                        &frame.view,
                        self.size.width,
                        self.size.height,
                    )
                    .expect("Draw queued");

                self.staging_belt.finish();
                self.queue.submit(Some(encoder.finish()));

                self.local_spawner
                    .spawn(self.staging_belt.recall())
                    .expect("Recall staging belt");

                self.local_pool.run_until_stalled();
            }

            _ => *control_flow = ControlFlow::Wait,
        }
    }

    pub fn new() -> Self {
        let event_loop = winit::event_loop::EventLoop::new();

        let window = winit::window::WindowBuilder::new()
            .with_resizable(false)
            .build(&event_loop)
            .unwrap();

        let instance = wgpu::Instance::new(wgpu::BackendBit::all());
        let surface = unsafe { instance.create_surface(&window) };

        // Initialize GPU
        let (device, queue) = futures::executor::block_on(async {
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: Some(&surface),
                })
                .await
                .expect("Request adapter");

            adapter
                .request_device(&wgpu::DeviceDescriptor::default(), None)
                .await
                .expect("Request device")
        });

        let staging_belt = wgpu::util::StagingBelt::new(1024);
        let local_pool = futures::executor::LocalPool::new();
        let local_spawner = local_pool.spawner();

        let size = window.inner_size();

        let swap_chain = device.create_swap_chain(
            &surface,
            &wgpu::SwapChainDescriptor {
                usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
                format: RENDER_FORMAT,
                width: size.width,
                height: size.height,
                present_mode: wgpu::PresentMode::Mailbox,
            },
        );

        let font = ab_glyph::FontArc::try_from_slice(include_bytes!("Inconsolata-Regular.ttf"))?;

        let glyph_brush = GlyphBrushBuilder::using_font(font).build(&device, RENDER_FORMAT);

        TextProgram {
            size,
            swap_chain,
            glyph_brush,
            surface,
            device,
            staging_belt,
            local_pool,
            local_spawner,
            queue,
            event_loop,
        }
    }

    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        self.window.request_redraw();

        self.event_loop.run(move |event, target, control_flow| {
            text_program.handle_event(event, target, control_flow)
        })
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let text_program = TextProgram::new();
    text_program.run()
}
