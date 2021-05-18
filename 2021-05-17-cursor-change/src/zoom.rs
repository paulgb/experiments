use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::window::{CursorIcon, Window};

type Mat4 = [f32; 16];

const ZOOM_FACTOR: f32 = 1.001;

pub struct ZoomState {
    center: (f32, f32),
    scale: (f32, f32),
    size: PhysicalSize<u32>,
    last_position: PhysicalPosition<f64>,
    dragging: bool,
}

impl ZoomState {
    pub fn new(size: PhysicalSize<u32>) -> Self {
        ZoomState {
            center: (0., 0.),
            scale: (1., 1.),
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
                let (x_scale, y_scale) = self.scale;
                let new_x_scale = x_scale * f32::powf(ZOOM_FACTOR, *y as f32);
                let new_y_scale = y_scale * f32::powf(ZOOM_FACTOR, *y as f32);

                self.scale = (new_x_scale, new_y_scale);

                // TODO: move center

                window.request_redraw();
                true
            }
            WindowEvent::CursorMoved { position, .. } => {
                if self.dragging {
                    let delta = (
                        position.x - self.last_position.x,
                        position.y - self.last_position.y,
                    );

                    let (x_center, y_center) = self.center;
                    self.center = (
                        x_center - delta.0 as f32 / self.scale.0 * 2.,
                        y_center + delta.1 as f32 / self.scale.1 * 2.,
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
        let x_x = self.scale.0 / self.size.width as f32;
        let y_y = -self.scale.1 / self.size.height as f32;
        let x_w = x_x * -self.center.0 as f32;
        let y_w = y_y * self.center.1 as f32;

        #[cfg_attr(rustfmt, rustfmt_skip)]
        [
            x_x,  0., 0., 0.,
             0., y_y, 0., 0.,
             0.,  0., 1., 0.,
            x_w, y_w, 0., 1.,
        ]
    }
}
