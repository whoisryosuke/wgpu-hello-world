use wgpu_hello_world::run;

fn main() {
    pollster::block_on(run());
}
