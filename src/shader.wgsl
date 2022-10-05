// Vertex shader

// Define any uniforms we expect from app
struct CameraUniform {
    view_proj: mat4x4<f32>,
};
// We create variables for the bind groups
// This is the "second" group we bound, so we access via `@group(1)`
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

// This is the input from the vertex buffer we created
// We get the properties from our Vertex struct here
// Note the index on location -- this relates to the properties placement in the buffer stride
// e.g. 0 = 1st "set" of data, 1 = 2nd "set"
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

// The output we send to our fragment shader
struct VertexOutput {
    // This property is "builtin" (aka used to render our vertex shader)
    @builtin(position) clip_position: vec4<f32>,
    // These are "custom" properties we can create to pass down
    // In this case, we pass the color down
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    // We define the output we want to send over to frag shader
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    // We set the "position" by using the `clip_position` property
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader

// We create variables for the bind groups 
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0)@binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // We use the special function `textureSample` to combine the texture data with coords
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}