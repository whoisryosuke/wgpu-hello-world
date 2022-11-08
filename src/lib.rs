use std::iter;

use cgmath::prelude::*;
use context::GraphicsContext;
use pass::phong::PhongPass;
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

mod camera;
mod context;
mod instance;
mod model;
mod pass;
mod resources;
mod texture;
mod window;
use crate::{
    camera::{Camera, CameraController, CameraUniform},
    context::create_render_pipeline,
    window::Window,
};
use crate::{
    instance::{Instance, InstanceRaw},
    window::WindowEvents,
};
use model::{DrawLight, DrawModel, Vertex};

// Constants for instances
const NUM_INSTANCES_PER_ROW: u32 = 10;

struct State {
    ctx: GraphicsContext,
    pass: PhongPass,
    // Window size
    size: winit::dpi::PhysicalSize<u32>,
    // Clear color for mouse interactions
    clear_color: wgpu::Color,
    // Cameraf
    camera: Camera,
    camera_controller: CameraController,
    // Instances
    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
    // 3D Model
    obj_model: model::Model,
}

impl State {
    // Initialize the state
    async fn new(window: &Window) -> Self {
        let size = window.window.inner_size();

        // Initialize the graphic context
        let ctx = GraphicsContext::new(&window).await;

        // Bind the camera to the shaders
        let camera = Camera {
            eye: (0.0, 5.0, -10.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: ctx.config.width as f32 / ctx.config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };
        let camera_controller = CameraController::new(0.2);

        // Initialize the pass
        let pass = PhongPass::new(&ctx.device, &ctx.queue, &ctx.config, &camera);

        // Create instance buffer
        // We create a 2x2 grid of objects by doing 1 nested loop here
        // And use the "displacement" matrix above to offset objects with a gap
        const SPACE_BETWEEN: f32 = 3.0;
        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                    let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

                    let position = cgmath::Vector3 { x, y: 0.0, z };

                    let rotation = if position.is_zero() {
                        cgmath::Quaternion::from_axis_angle(
                            cgmath::Vector3::unit_z(),
                            cgmath::Deg(0.0),
                        )
                    } else {
                        cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                    };

                    Instance { position, rotation }
                })
            })
            .collect::<Vec<_>>();

        // We condense the matrix properties into a flat array (aka "raw data")
        // (which is how buffers work - so we can "stride" over chunks)
        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        // Create the instance buffer with our data
        let instance_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX,
            });

        // Load model from disk or as a HTTP request (for web support)
        log::warn!("Load model");
        let obj_model = resources::load_model(
            "banana.obj",
            &ctx.device,
            &ctx.queue,
            &pass.texture_bind_group_layout,
        )
        .await
        .expect("Couldn't load model. Maybe path is wrong?");

        // Clear color used for mouse input interaction
        let clear_color = wgpu::Color::BLACK;

        Self {
            ctx,
            pass,
            clear_color,
            size,
            camera,
            camera_controller,
            instances,
            instance_buffer,
            obj_model,
        }
    }

    // Keeps state in sync with window size when changed
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.ctx.config.width = new_size.width;
            self.ctx.config.height = new_size.height;
            self.ctx
                .surface
                .configure(&self.ctx.device, &self.ctx.config);
            // Make sure to current window size to depth texture - required for calc
            self.pass.depth_texture = texture::Texture::create_depth_texture(
                &self.ctx.device,
                &self.ctx.config,
                "depth_texture",
            );
        }
    }

    // Handle input using WindowEvent
    fn input(&mut self, event: &WindowEvent) -> bool {
        // Send any input to camera controller
        self.camera_controller.process_events(event);

        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.clear_color = wgpu::Color {
                    r: 0.0,
                    g: position.y as f64 / self.size.height as f64,
                    b: position.x as f64 / self.size.width as f64,
                    a: 1.0,
                };
                true
            }
            _ => false,
        }
    }

    fn update(&mut self) {
        // Sync local app state with camera
        self.camera_controller.update_camera(&mut self.camera);
        self.pass.camera_uniform.update_view_proj(&self.camera);
        self.ctx.queue.write_buffer(
            &self.pass.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.pass.camera_uniform]),
        );

        // Update the light
        let old_position: cgmath::Vector3<_> = self.pass.light_uniform.position.into();
        self.pass.light_uniform.position =
            (cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(1.0))
                * old_position)
                .into();
        self.ctx.queue.write_buffer(
            &self.pass.light_buffer,
            0,
            bytemuck::cast_slice(&[self.pass.light_uniform]),
        );
    }

    // Primary render flow
    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.ctx.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // Set the clear color during redraw
                        // This is basically a background color applied if an object isn't taking up space

                        // This sets it a color that changes based on mouse move
                        // load: wgpu::LoadOp::Clear(self.clear_color),

                        // A standard clear color
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                // Create a depth stencil buffer using the depth texture
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.pass.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            // Setup our render pipeline with our config earlier in `new()`
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

            // Setup lighting pipeline
            render_pass.set_pipeline(&self.pass.light_render_pipeline);
            // Draw/calculate the lighting on models
            render_pass.draw_light_model(
                &self.obj_model,
                &self.pass.camera_bind_group,
                &self.pass.light_bind_group,
            );

            // Setup render pipeline
            render_pass.set_pipeline(&self.pass.render_pipeline);
            // Draw the models
            render_pass.draw_model_instanced(
                &self.obj_model,
                0..self.instances.len() as u32,
                &self.pass.camera_bind_group,
                &self.pass.light_bind_group,
            );
        }

        self.ctx.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let window = Window::new();

    // @TODO: Put inside Window module
    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        window.window.set_inner_size(PhysicalSize::new(450, 400));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.body()?;
                let canvas = web_sys::Element::from(window.window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    // State::new uses async code, so we're going to wait for it to finish
    let mut state = State::new(&window).await;

    // @TODO: Wire up state methods again (like render)
    window.run(move |event| match event {
        WindowEvents::Resized { width, height } => {
            state.resize(winit::dpi::PhysicalSize { width, height });
        }
        WindowEvents::Draw => {
            state.update();
            state.render();
        }
        WindowEvents::Keyboard {
            state,
            virtual_keycode,
        } => {}
    });
}
