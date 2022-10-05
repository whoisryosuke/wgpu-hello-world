// Vertex shader

// This is the input from the vertex buffer we created
// We get the properties from our Vertex struct here
// Note the index on location -- this relates to the properties placement in the buffer stride
// e.g. 0 = 1st "set" of data, 1 = 2nd "set"
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

// The output we send to our fragment shader
struct VertexOutput {
    // This property is "builtin" (aka used to render our vertex shader)
    @builtin(position) clip_position: vec4<f32>,
    // These are "custom" properties we can create to pass down
    // In this case, we pass the color down
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    // We define the output we want to send over to frag shader
    var out: VertexOutput;
    out.color = model.color;
    // We set the "position" by using the `clip_position` property
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // We grab the color passed down from the vertex shader
    return vec4<f32>(in.color, 1.0);
}