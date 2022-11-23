// Vertex shader
// Define any uniforms we expect from app
struct Globals {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
    ambient: vec4<f32>,
};
struct Locals {
    position:  vec4<f32>,
    color:  vec4<f32>,
    normal:  vec4<f32>,
    lights:  vec4<f32>,
}
// We create variables for the bind groups
@group(0) @binding(0)
var<uniform> globals: Globals;
@group(1) @binding(0)
var<uniform> locals: Locals;

struct Light {
    position: vec3<f32>,
    color: vec3<f32>,
}
@group(0) @binding(1)
var<uniform> light: Light;

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    let scale = 0.25;
    var out: VertexOutput;
    out.clip_position = globals.view_proj * vec4<f32>(model.position * scale + light.position, 1.0);
    out.color = light.color;
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}