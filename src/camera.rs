use cgmath::prelude::*;

use winit::{dpi::PhysicalPosition, event::*};

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        proj * view
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_position: [0.0; 4],
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        // We're using Vector4 because ofthe camera_uniform 16 byte spacing requirement
        self.view_position = camera.eye.to_homogeneous().into();
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

pub struct CameraController {
    speed: f32,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_mouse_pressed: bool,

    prev_position: PhysicalPosition<f64>,
    position: PhysicalPosition<f64>,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_up_pressed: false,
            is_down_pressed: false,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_mouse_pressed: false,

            prev_position: PhysicalPosition { x: 0.0, y: 0.0 },
            position: PhysicalPosition { x: 0.0, y: 0.0 },
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            // Check for mouse input
            WindowEvent::MouseInput { state, button, .. } => match button {
                MouseButton::Left => {
                    match state {
                        ElementState::Pressed => {
                            println!("Mouse pressed");
                            self.is_mouse_pressed = true;
                        }
                        ElementState::Released => {
                            println!("Mouse unpressed");
                            self.is_mouse_pressed = false;
                            self.prev_position = PhysicalPosition { x: 0.0, y: 0.0 };
                            self.position = PhysicalPosition { x: 0.0, y: 0.0 };
                        }
                    }
                    true
                }
                _ => false,
            },
            WindowEvent::CursorMoved { position, .. } => {
                // self.clear_color = wgpu::Color {
                //     r: 0.0,
                //     g: position.y as f64 / self.size.height as f64,
                //     b: position.x as f64 / self.size.width as f64,
                //     a: 1.0,
                // };
                // Mouse pressed? Track mouse movement for dragging
                if self.is_mouse_pressed {
                    // Do we have initial position?
                    if self.prev_position.x == 0.0 {
                        self.prev_position = position.clone();
                    } else {
                        self.prev_position = self.position.clone();
                    }

                    // Save current mouse position
                    self.position = position.clone();
                    println!("Mouse pressed - recording position")
                }
                true
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::Space => {
                        self.is_up_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::LShift => {
                        self.is_down_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::W | VirtualKeyCode::Up => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::A | VirtualKeyCode::Left => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::S | VirtualKeyCode::Down => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::D | VirtualKeyCode::Right => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();
        let x_dist = ((self.prev_position.x - self.position.x) / 100.0) as f32;
        let y_dist = ((self.prev_position.y - self.position.y) / 100.0) as f32;

        // Handle right click input (zoom in/out for up/down)
        // Checking for forward_mag prevents glitching when camera
        // gets too close to the center of the scene.
        if self.is_mouse_pressed && forward_mag > y_dist && y_dist > 0.0 {
            println!("Going up!");
            camera.eye += forward_norm * y_dist.abs();
        }
        if self.is_mouse_pressed && y_dist < 0.0 {
            println!("Going down!");
            camera.eye -= forward_norm * y_dist.abs();
        }

        let right = forward_norm.cross(camera.up);

        // Redo radius calc in case the up/ down is pressed.
        let forward = camera.target - camera.eye;
        let forward_mag = forward.magnitude();

        if self.is_mouse_pressed {
            dbg!(x_dist);
            dbg!(y_dist);

            // Horizontal + Vertical movement
            let horizontal_movement = (forward + right * x_dist).normalize();
            // let vertical_movement = (forward + camera.up * y_dist).normalize();
            camera.eye = camera.target - horizontal_movement * forward_mag;
        }
        if self.is_right_pressed {
            // Rescale the distance between the target and eye so
            // that it doesn't change. The eye therefore still
            // lies on the circle made by the target and eye.
            camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;
        }
        if self.is_left_pressed {
            camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
        }
    }
}
