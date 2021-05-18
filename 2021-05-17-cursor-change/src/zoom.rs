use cgmath::Vector2;
use cgmath::{ElementWise, Point2};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::window::{CursorIcon, Window};

type Mat4 = [f32; 16];

const ZOOM_FACTOR: f32 = 1.001;

fn size_to_vec(size: PhysicalSize<u32>) -> Vector2<f32> {
    Vector2::new(size.width as f32, size.height as f32)
}

fn position_to_vec(size: PhysicalPosition<u32>) -> Vector2<f32> {
    Vector2::new(size.x as f32, size.y as f32)
}

#[derive(Debug)]
struct WindowCoordinate(PhysicalPosition<u32>);

#[derive(Debug, Clone, Copy)]
struct SceneCoordinate(pub Vector2<f32>);

#[derive(Debug)]
struct GPUCoordinate(pub Vector2<f32>);

impl WindowCoordinate {
    pub fn to_gpu_coordinate(&self, size: PhysicalSize<u32>) -> GPUCoordinate {
        let coordinate = position_to_vec(self.0);
        GPUCoordinate(
            2. * (ElementWise::div_element_wise(coordinate, size_to_vec(size)))
                - Vector2::new(1., 1.),
        )
    }
}

impl GPUCoordinate {
    pub fn to_scene_coordinate(
        &self,
        center: SceneCoordinate,
        scale: Vector2<f32>,
        size: PhysicalSize<u32>,
    ) -> SceneCoordinate {
        let GPUCoordinate(coordinate) = *self;
        SceneCoordinate(
            ElementWise::mul_element_wise(
                ElementWise::div_element_wise(size_to_vec(size), scale),
                coordinate,
            ) + center.0,
        )
    }
}

pub struct ZoomState {
    center: SceneCoordinate,
    scale: Vector2<f32>,
    size: PhysicalSize<u32>,
    last_position: PhysicalPosition<f64>,
    dragging: bool,
}

impl ZoomState {
    pub fn new(size: PhysicalSize<u32>) -> Self {
        ZoomState {
            center: SceneCoordinate(Vector2::new(0., 0.)),
            scale: Vector2::new(1., 1.),
            size,
            last_position: PhysicalPosition { x: 0., y: 0. },
            dragging: false,
        }
    }

    pub fn set_size(&mut self, size: PhysicalSize<u32>) {
        self.size = size;
    }

    pub fn handle_event(&mut self, event: &WindowEvent, window: &Window) -> bool {
        match event {
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                self.dragging = match state {
                    ElementState::Pressed => {
                        window.set_cursor_icon(CursorIcon::Grabbing);
                        true
                    }
                    ElementState::Released => {
                        window.set_cursor_icon(CursorIcon::Arrow);
                        false
                    }
                };
                true
            }
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::PixelDelta(PhysicalPosition { y, .. }),
                ..
            } => {
                let zoom_multiplier = f32::powf(ZOOM_FACTOR, *y as f32);

                self.scale *= zoom_multiplier;

                window.request_redraw();
                true
            }
            WindowEvent::CursorMoved { position, .. } => {
                if self.dragging {
                    let delta = Vector2::new(
                        self.last_position.x as f32 - position.x as f32,
                        position.y as f32 - self.last_position.y as f32,
                    );

                    self.center = SceneCoordinate(
                        self.center.0 + 2. * ElementWise::div_element_wise(delta, self.scale),
                    );

                    window.request_redraw();
                }

                self.last_position = position.clone();
                true
            }
            _ => false,
        }
    }

    pub fn matrix(&self) -> Mat4 {
        let x_x = self.scale[0] / self.size.width as f32;
        let y_y = self.scale[1] / self.size.height as f32;
        let x_w = x_x * -self.center.0[0] as f32;
        let y_w = y_y * -self.center.0[1] as f32;

        #[cfg_attr(rustfmt, rustfmt_skip)]
        [
            x_x,  0., 0., 0.,
             0., y_y, 0., 0.,
             0.,  0., 1., 0.,
            x_w, y_w, 0., 1.,
        ]
    }
}
