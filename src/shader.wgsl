// Vertex shader

// Define the vertex output
// This gets sent to the fragment shader as a parameter (see `in` below)
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    // We create a new 2D vector to send position coordinates to fragment shader
    @location(0) position: vec2<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    // Position vertices in a triangle shape
    // This is calculated using the index of the vertex (0-2) and a math equation
    var out: VertexOutput;
    let x = f32(1 - i32(in_vertex_index)) * 0.5;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;

    // We pass the position we created to the vertex output `struct`
    // These get passed to fragment shader and also used by renderer (like position)
    out.position = vec2<f32>(x, y);
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // We use the position coordinates from vertex shader above as colors
    return vec4<f32>(in.position, 0.5, 1.0);
}