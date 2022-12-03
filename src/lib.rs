use std::{iter, time::Instant};

use cgmath::prelude::*;
use context::GraphicsContext;
use node::Node;
use pass::{phong::PhongPass, Pass};
use wgpu::util::DeviceExt;
use winit::{
    dpi::PhysicalPosition,
    event::*,
    event_loop::{ControlFlow, EventLoop},
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

mod camera;
mod context;
mod instance;
mod model;
mod node;
mod pass;
mod primitives;
mod resources;
mod texture;
mod window;
use crate::{
    camera::{Camera, CameraController, CameraUniform},
    context::create_render_pipeline,
    model::Keyframes,
    pass::phong::{Locals, PhongConfig},
    primitives::{sphere::generate_sphere, PrimitiveMesh},
    window::Window,
};
use crate::{
    instance::{Instance, InstanceRaw},
    window::WindowEvents,
};
use model::{DrawLight, DrawModel, Vertex};

struct State {
    ctx: GraphicsContext,
    pass: PhongPass,
    // Window size
    size: winit::dpi::PhysicalSize<u32>,
    // Clear color for mouse interactions
    clear_color: wgpu::Color,
    // Camera
    camera: Camera,
    camera_controller: CameraController,
    // The 3D models in the scene (as Nodes)
    nodes: Vec<Node>,
    // Animation
    time: Instant,
}

impl State {
    // Initialize the state
    async fn new(window: &Window) -> Self {
        // Save the window size for use later
        let size = window.window.inner_size();

        // Initialize the graphic context
        let ctx = GraphicsContext::new(&window).await;

        // Setup the camera and it's initial position
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
        let pass_config = PhongConfig {
            max_lights: 1,
            ambient: Default::default(),
            wireframe: false,
        };
        let pass = PhongPass::new(&pass_config, &ctx.device, &ctx.queue, &ctx.config, &camera);

        // Create the 3D objects!
        // Load 3D model from disk or as a HTTP request (for web support)
        log::warn!("Load model");
        let obj_model = resources::load_model("banana.obj", &ctx.device, &ctx.queue)
            .await
            .expect("Couldn't load model. Maybe path is wrong?");
        let ferris_model = resources::load_model("ferris.obj", &ctx.device, &ctx.queue)
            .await
            .expect("Couldn't load model. Maybe path is wrong?");
        let gltf_model =
            resources::load_model_gltf("Cube-Tris-Textured-Animated.gltf", &ctx.device, &ctx.queue)
                .await
                .expect("Couldn't load model. Maybe path is wrong?");

        let cube_primitive = PrimitiveMesh::new(
            &ctx.device,
            &ctx.queue,
            &primitives::cube::cube_vertices(0.5),
            &primitives::cube::cube_indices(),
        )
        .await;

        let plane_primitive = PrimitiveMesh::new(
            &ctx.device,
            &ctx.queue,
            &primitives::plane::plane_vertices(0.5),
            &primitives::plane::plane_indices(),
        )
        .await;

        let (sphere_vertices, sphere_indices) = generate_sphere(2.0, 36, 18);

        let sphere_primitive =
            PrimitiveMesh::new(&ctx.device, &ctx.queue, &sphere_vertices, &sphere_indices).await;

        // Create instances for each object with locational data (position + rotation)
        // Renderer currently defaults to using instances. Want one object? Pass a Vec of 1 instance.

        // We create a 2x2 grid of objects by doing 1 nested loop here
        // And use the "displacement" matrix above to offset objects with a gap
        const SPACE_BETWEEN: f32 = 3.0;
        const NUM_INSTANCES_PER_ROW: u32 = 10;
        let banana_instances = (0..NUM_INSTANCES_PER_ROW)
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

        // More "manual" placement as an example
        let ferris_instances = (0..2)
            .map(|z| {
                let z = SPACE_BETWEEN * (z as f32);
                let position = cgmath::Vector3 { x: z, y: 1.0, z };
                let rotation = if position.is_zero() {
                    cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
                } else {
                    cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                };
                Instance { position, rotation }
            })
            .collect::<Vec<_>>();

        // The cube primitive instances (aka positions)
        let cube_primitive_instances = (0..2)
            .map(|z| {
                let z = SPACE_BETWEEN * (z as f32);
                let position = cgmath::Vector3 {
                    x: -z + 2.0,
                    y: 2.0,
                    z: -z,
                };
                let rotation = if position.is_zero() {
                    cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
                } else {
                    cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                };
                Instance { position, rotation }
            })
            .collect::<Vec<_>>();

        let plane_primitive_instances = vec![Instance {
            position: cgmath::Vector3 {
                x: -3.0,
                y: 3.0,
                z: -3.0,
            },
            rotation: cgmath::Quaternion::from_axis_angle(
                cgmath::Vector3::unit_z(),
                cgmath::Deg(0.0),
            ),
        }];

        let sphere_primitive_instances = vec![Instance {
            position: cgmath::Vector3 {
                x: 3.0,
                y: 3.0,
                z: 3.0,
            },
            rotation: cgmath::Quaternion::from_axis_angle(
                cgmath::Vector3::unit_z(),
                cgmath::Deg(0.0),
            ),
        }];

        // Create the nodes
        let banana_node = Node {
            parent: 0,
            locals: Locals {
                position: [0.0, 0.0, 0.0, 0.0],
                color: [0.0, 0.0, 1.0, 1.0],
                normal: [0.0, 0.0, 0.0, 0.0],
                lights: [0.0, 0.0, 0.0, 0.0],
            },
            model: obj_model,
            instances: banana_instances,
        };

        let ferris_node = Node {
            parent: 0,
            locals: Locals {
                position: [0.0, 0.0, 0.0, 0.0],
                color: [0.0, 0.0, 1.0, 1.0],
                normal: [0.0, 0.0, 0.0, 0.0],
                lights: [0.0, 0.0, 0.0, 0.0],
            },
            model: ferris_model,
            instances: ferris_instances,
        };

        let cube_primitive_node = Node {
            parent: 0,
            locals: Locals {
                position: [0.0, 0.0, 0.0, 0.0],
                color: [0.0, 0.0, 1.0, 1.0],
                normal: [0.0, 0.0, 0.0, 0.0],
                lights: [0.0, 0.0, 0.0, 0.0],
            },
            model: cube_primitive.model,
            instances: cube_primitive_instances,
        };

        let plane_primitive_node = Node {
            parent: 0,
            locals: Locals {
                position: [0.0, 0.0, 0.0, 0.0],
                color: [0.0, 0.0, 1.0, 1.0],
                normal: [0.0, 0.0, 0.0, 0.0],
                lights: [0.0, 0.0, 0.0, 0.0],
            },
            model: gltf_model,
            instances: plane_primitive_instances,
        };

        let sphere_primitive_node = Node {
            parent: 0,
            locals: Locals {
                position: [0.0, 0.0, 0.0, 0.0],
                color: [0.0, 0.0, 1.0, 1.0],
                normal: [0.0, 0.0, 0.0, 0.0],
                lights: [0.0, 0.0, 0.0, 0.0],
            },
            model: sphere_primitive.model,
            instances: sphere_primitive_instances,
        };

        // Put all our nodes into an Vector to loop over later
        let nodes = vec![
            banana_node,
            ferris_node,
            cube_primitive_node,
            plane_primitive_node,
            sphere_primitive_node,
        ];

        // Clear color used for mouse input interaction
        let clear_color = wgpu::Color::BLACK;

        let time = Instant::now();

        Self {
            ctx,
            pass,
            clear_color,
            size,
            camera,
            camera_controller,
            nodes,
            time,
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
    pub fn keyboard(&mut self, state: ElementState, keycode: &VirtualKeyCode) -> bool {
        // Send any input to camera controller
        self.camera_controller.process_events(&state, &keycode)

        // match event {
        //     WindowEvent::CursorMoved { position, .. } => {
        //         self.clear_color = wgpu::Color {
        //             r: 0.0,
        //             g: position.y as f64 / self.size.height as f64,
        //             b: position.x as f64 / self.size.width as f64,
        //             a: 1.0,
        //         };
        //         true
        //     }
        //     _ => false,
        // }
    }

    pub fn mouse_moved(&mut self, position: &PhysicalPosition<f64>) {
        self.camera_controller
            .process_mouse_moved(&position, &self.size);
    }
    pub fn mouse_input(
        &mut self,
        device_id: &DeviceId,
        state: &ElementState,
        button: &MouseButton,
    ) {
        self.camera_controller
            .process_mouse_input(device_id, state, button);
    }

    fn update(&mut self) {
        // Sync local app state with camera
        self.camera_controller.update_camera(&mut self.camera);
        self.pass.camera_uniform.update_view_proj(&self.camera);
        self.ctx.queue.write_buffer(
            &self.pass.global_uniform_buffer,
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

        println!("Time elapsed: {:?}", &self.time.elapsed());

        // Update local uniforms
        let current_time = &self.time.elapsed().as_secs_f32();
        let mut node_index = 0;
        for node in &mut self.nodes {
            // Play animations
            if node.model.animations.len() > 0 {
                // Loop through all animations
                // TODO: Ideally we'd play a certain animation by name - we assume first one for now
                let mut current_keyframe_index = 0;
                for animation in &node.model.animations {
                    for timestamp in &animation.timestamps {
                        if timestamp > current_time {
                            break;
                        }
                        if &current_keyframe_index < &(&animation.timestamps.len() - 1) {
                            current_keyframe_index += 1;
                        }
                    }
                }

                // Update locals with current animation
                let current_animation = &node.model.animations[0].keyframes;
                let mut current_frame: Option<&Vec<f32>> = None;
                match current_animation {
                    Keyframes::Translation(frames) => {
                        current_frame = Some(&frames[current_keyframe_index])
                    }
                    Keyframes::Other => (),
                }

                if current_frame.is_some() {
                    let current_frame = current_frame.unwrap();

                    node.locals.position = [
                        current_frame[0],
                        current_frame[1],
                        current_frame[2],
                        node.locals.position[3],
                    ];
                }
            }

            // node.locals.position = [
            //     node.locals.position[0],
            //     (node.locals.position[1] + 0.001),
            //     (node.locals.position[2] - 0.001),
            //     node.locals.position[3],
            // ];
            &self
                .pass
                .uniform_pool
                .update_uniform(node_index, node.locals, &self.ctx.queue);
            node_index += 1;
        }
    }

    // Primary render flow
    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        match self.pass.draw(
            &self.ctx.surface,
            &self.ctx.device,
            &self.ctx.queue,
            &self.nodes,
        ) {
            Err(err) => println!("Error in rendering"),
            Ok(_) => (),
        }

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
    let mut app = State::new(&window).await;

    // @TODO: Wire up state methods again (like render)
    window.run(move |event| match event {
        WindowEvents::Resized { width, height } => {
            app.resize(winit::dpi::PhysicalSize { width, height });
        }
        WindowEvents::Draw => {
            app.update();
            match app.render() {
                Err(err) => println!("Error in rendering"),
                Ok(_) => (),
            }
        }
        WindowEvents::Keyboard {
            state,
            virtual_keycode,
        } => {
            app.keyboard(state, virtual_keycode);
        }

        WindowEvents::MouseMoved { position } => {
            app.mouse_moved(position);
        }

        WindowEvents::MouseInput {
            device_id,
            state,
            button,
        } => {
            app.mouse_input(device_id, state, button);
        }
    });
}
