use cgmath::{prelude::*, Point3};

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
    // Keyboard input
    is_up_pressed: bool,
    is_modifier_shift_pressed: bool,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,

    // Mouse input
    is_mouse_right_pressed: bool,
    is_mouse_right_tracked: bool,

    // Mouse position
    // The initial or previous position, used for calculating direction/speed of movement
    mouse_initial_position: PhysicalPosition<f32>,
    // The difference between initial + current position
    mouse_diff_position: PhysicalPosition<f32>,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_up_pressed: false,
            is_modifier_shift_pressed: false,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_mouse_right_pressed: false,
            is_mouse_right_tracked: false,
            mouse_initial_position: PhysicalPosition { x: 0.0, y: 0.0 },
            mouse_diff_position: PhysicalPosition { x: 0.0, y: 0.0 },
        }
    }

    /// Handle keyboard input for camera (like moving camera with WASD keys)
    pub fn process_events(
        &mut self,
        state: &ElementState,
        &virtual_keycode: &VirtualKeyCode,
    ) -> bool {
        let is_pressed = *state == ElementState::Pressed;
        match virtual_keycode {
            VirtualKeyCode::Space => {
                self.is_up_pressed = is_pressed;
                true
            }
            VirtualKeyCode::LShift => {
                self.is_modifier_shift_pressed = is_pressed;
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

    /// Handle mouse input for camera (like moving camera based on mouse position)
    pub fn process_mouse_moved(
        &mut self,
        position: &PhysicalPosition<f64>,
        screen_size: &winit::dpi::PhysicalSize<u32>,
    ) {
        println!(
            "Mouse position X: {} - Y : {}",
            &position.x / screen_size.width as f64,
            &position.y / screen_size.height as f64
        );

        let current_x = &position.x / screen_size.width as f64;
        let current_y = &position.y / screen_size.height as f64;

        // Not tracking? Set initial position
        if self.is_mouse_right_pressed && !self.is_mouse_right_tracked {
            self.mouse_initial_position = PhysicalPosition {
                x: current_x as f32,
                y: current_y as f32,
            };
            self.is_mouse_right_tracked = true;
        }

        // Tracking? Set current position
        if self.is_mouse_right_pressed && self.is_mouse_right_tracked {
            self.mouse_diff_position = PhysicalPosition {
                x: current_x as f32 - self.mouse_initial_position.x,
                y: current_y as f32 - self.mouse_initial_position.y,
            };
        }

        // Not pressing anymore? Stop tracking.
        if !self.is_mouse_right_pressed && self.is_mouse_right_tracked {
            self.is_mouse_right_tracked = false;
        }
    }

    pub fn process_mouse_input(
        &mut self,
        device_id: &DeviceId,
        state: &ElementState,
        button: &MouseButton,
    ) {
        // println!("MOUSE INPUT");
        // println!("Device ID:");
        // dbg!(device_id);
        // println!("State:");
        // dbg!(state);
        // println!("Button:");
        // dbg!(button);

        match button {
            MouseButton::Right => {
                self.is_mouse_right_pressed = *state == ElementState::Pressed;
            }
            // MouseButton::Left => todo!(),
            // MouseButton::Middle => todo!(),
            _ => (),
        }
    }

    /// The render loop for camera. Updates camera position every frame (or fn call).
    pub fn update_camera(&self, camera: &mut Camera) {
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        // Prevents glitching when camera gets too close to the
        // center of the scene.
        if self.is_forward_pressed && forward_mag > self.speed {
            camera.eye += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * self.speed;
        }

        let right = forward_norm.cross(camera.up);

        // Redo radius calc in case the up/ down is pressed.
        let forward = camera.target - camera.eye;
        let forward_mag = forward.magnitude();

        // Keyboard input
        if self.is_right_pressed {
            // Rescale the distance between the target and eye so
            // that it doesn't change. The eye therefore still
            // lies on the circle made by the target and eye.
            // camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;

            camera.eye = camera.target - (forward - right * self.speed);
            // Move the target up
            camera.target += right * self.speed;
        }

        if self.is_left_pressed {
            // camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
            camera.eye = camera.target - (forward + right * self.speed);
            // Move the target up
            camera.target -= right * self.speed;
        }

        // Left shift pressed
        if self.is_modifier_shift_pressed {
            if self.is_up_pressed {
                // Move the character down in the Z space (like jumping up)
                // Move the eye up (but stay focused on target)
                camera.eye = camera.target - (forward + camera.up * self.speed);
                // Move the target up
                camera.target -= camera.up * self.speed;
            }
        }

        // Shift actions that need default state
        if !self.is_modifier_shift_pressed {
            if self.is_up_pressed {
                // "rotate around up"
                // camera.eye =
                // camera.target - (forward - camera.up * self.speed).normalize() * forward_mag;

                // Move the character up in the Z space (like jumping up)
                // Move the eye up (but stay focused on target)
                camera.eye = camera.target - (forward - camera.up * self.speed);
                // Move the target up
                camera.target += camera.up * self.speed;
            }
        }

        // Mouse input
        if self.is_mouse_right_tracked {
            // Rotate camera based on mouse movement.
            // We take difference of initial pos and current pos
            // and use that as base vector in rotation calculations
            // We use the X for left/right and Y for up/down calcs.

            camera.eye = camera.target - (forward + right * self.mouse_diff_position.x);
            // camera.eye = camera.target - (forward - camera.up * self.mouse_diff_position.y);
        }
    }
}
