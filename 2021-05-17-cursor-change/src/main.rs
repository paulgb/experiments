use std::iter;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::line::{Line, LinesLayer};
use crate::rectangle::{Rectangle, RectanglesLayer};
use circle::{Circle, CirclesLayer};
use layer::{Drawable, Layer};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsage, ShaderStage,
};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::window::CursorIcon;

mod circle;
mod layer;
mod line;
mod rectangle;

type Mat4 = [f32; 16];

pub fn transformation_matrix(width: u32, height: u32, x_offset: f64, y_offset: f64) -> Mat4 {
    let x_x = 2. / width as f32;
    let y_y = -2. / height as f32;
    let x_w = -1. + (x_offset as f32 / width as f32);
    let y_w = 1. - (y_offset as f32 / height as f32);

    #[cfg_attr(rustfmt, rustfmt_skip)]
    [
        x_x,  0., 0., 0.,
         0., y_y, 0., 0.,
         0.,  0., 1., 0.,
        x_w, y_w, 0., 1.,
    ]
}

#[derive(PartialEq)]
enum DragState {
    NotDragging,
    MouseDown,
    DraggedFrom(PhysicalPosition<f64>),
}

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    size: winit::dpi::PhysicalSize<u32>,
    transform: Mat4,
    transform_buffer: Buffer,
    transform_bind_group: BindGroup,

    drawables: Vec<Box<dyn Drawable>>,
    drag_state: DragState,
    offset: (f64, f64),
}

impl State {
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
                None, // Trace path
            )
            .await
            .unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let layers: Vec<Box<dyn Layer>> = vec![
            Box::new(CirclesLayer::new(vec![
                Circle {
                    position: [20., 20.],
                    radius: 15.,
                    color: [0.1, 1.0, 0.5, 1.],
                },
                Circle {
                    position: [300., 300.],
                    radius: 50.,
                    color: [0.6, 0.6, 0., 1.],
                },
                Circle {
                    position: [350., 350.],
                    radius: 70.,
                    color: [0.7, 0., 0.4, 1.],
                },
            ])),
            Box::new(CirclesLayer::new(vec![Circle {
                position: [500., 300.],
                radius: 40.,
                color: [0.3, 0.6, 0.9, 1.],
            }])),
            Box::new(RectanglesLayer::new(vec![
                Rectangle {
                    upper_left: [400., 400.],
                    bottom_right: [450., 500.],
                    color: [0.3, 0.6, 0.4, 1.],
                },
                Rectangle {
                    upper_left: [10., 250.],
                    bottom_right: [50., 300.],
                    color: [0.7, 0., 0.4, 1.],
                },
            ])),
            Box::new(LinesLayer::new(vec![Line {
                start: [450., 450.],
                end: [200., 100.],
                width: 3.,
                color: [0.0, 0.0, 0.0, 1.0],
            }])),
        ];

        let transform = transformation_matrix(size.width, size.height, 0., 0.);

        let transform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Transformation buffer"),
            contents: bytemuck::cast_slice(&[transform]),
            usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
        });

        let transform_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Transformation bind group layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStage::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let transform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Transformation bind group"),
            layout: &transform_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: transform_buffer.as_entire_binding(),
            }],
        });

        let drawables = layers
            .into_iter()
            .map(|d| d.init_drawable(&device, &sc_desc, &transform_layout))
            .collect();

        Self {
            surface,
            device,
            queue,
            size,
            sc_desc,
            swap_chain,
            drawables,
            transform,
            transform_buffer,
            transform_bind_group,
            drag_state: DragState::NotDragging,
            offset: (0., 0.),
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;

        let (x_offset, y_offset) = self.offset;
        self.transform = transformation_matrix(new_size.width, new_size.height, x_offset, y_offset);

        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    fn input(&mut self, event: &WindowEvent, window: &Window) -> bool {
        match event {
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                self.drag_state = match state {
                    ElementState::Pressed => {
                        window.set_cursor_icon(CursorIcon::Grabbing);
                        DragState::MouseDown
                    },
                    ElementState::Released => {
                        window.set_cursor_icon(CursorIcon::Arrow);
                        DragState::NotDragging
                    },
                };
                true
            }
            WindowEvent::CursorMoved { position, .. } => {
                if self.drag_state == DragState::NotDragging {
                    false
                } else {
                    if let DragState::DraggedFrom(last_position) = self.drag_state {
                        let delta = (position.x - last_position.x, position.y - last_position.y);

                        let (x_offset, y_offset) = self.offset;
                        self.offset = (x_offset + delta.0, y_offset + delta.1);

                        let (x_offset, y_offset) = self.offset;
                        self.transform = transformation_matrix(self.size.width, self.size.height, x_offset, y_offset);
                    }

                    self.drag_state = DragState::DraggedFrom(*position);

                    window.request_redraw();
                    true
                }
            }
            _ => false,
        }
    }

    fn update(&mut self) {}

    fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        println!("render");
        let frame = self.swap_chain.get_current_frame()?.output;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.,
                            g: 1.,
                            b: 1.,
                            a: 1.,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            self.queue.write_buffer(
                &self.transform_buffer,
                0,
                &bytemuck::cast_slice(&self.transform),
            );

            for drawable in &self.drawables {
                drawable.draw(&mut render_pass, &self.transform_bind_group);
            }
        }

        self.queue.submit(iter::once(encoder.finish()));

        Ok(())
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Shape Drawing Demo")
        .with_inner_size(PhysicalSize {
            width: 600,
            height: 600,
        })
        .build(&event_loop)
        .unwrap();

    use futures::executor::block_on;

    // Since main can't be async, we're going to need to block
    let mut state = block_on(State::new(&window));

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !state.input(event, &window) {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::KeyboardInput {
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
                            // new_inner_size is &mut so w have to dereference it twice
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(_) => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Recreate the swap_chain if lost
                    Err(wgpu::SwapChainError::Lost) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                //window.request_redraw();
            }
            _ => {}
        }
    });
}
