use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window,
};

pub struct Window {
    pub event_loop: EventLoop<()>,
    pub window: window::Window,
}

impl Window {
    pub fn new() -> Self {
        // TODO: Add size
        let event_loop = EventLoop::new();
        let window = window::WindowBuilder::new()
            .with_title("ryos wgpu playground")
            .build(&event_loop)
            .unwrap();

        Self { event_loop, window }
    }
}
