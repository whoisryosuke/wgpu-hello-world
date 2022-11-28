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

// This is the input from the vertex buffer we created
// We get the properties from our Vertex struct here
// Note the index on location -- this relates to the properties placement in the buffer stride
// e.g. 0 = 1st "set" of data, 1 = 2nd "set"
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
};
// The instance buffer
struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
    @location(9) normal_matrix_0: vec3<f32>,
    @location(10) normal_matrix_1: vec3<f32>,
    @location(11) normal_matrix_2: vec3<f32>,
}

// The output we send to our fragment shader
struct VertexOutput {
    // This property is "builtin" (aka used to render our vertex shader)
    @builtin(position) clip_position: vec4<f32>,
    // These are "custom" properties we can create to pass down
    // In this case, we pass the color down
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    // Reconstruct the matrix from the flattened/raw data
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    let normal_matrix = mat3x3<f32>(
        instance.normal_matrix_0,
        instance.normal_matrix_1,
        instance.normal_matrix_2,
    );

    // We define the output we want to send over to frag shader
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;

    out.world_normal = normal_matrix * model.normal;
    var world_position: vec4<f32> = model_matrix * (vec4<f32>(model.position, 1.0) + locals.position);
    out.world_position = world_position.xyz;

    // We set the "position" by using the `clip_position` property
    // We multiply it by the camera position matrix and the instance position matrix
    out.clip_position = globals.view_proj * model_matrix * (vec4<f32>(model.position, 1.0) + locals.position);
    return out;
}

// Fragment shader

// We create variables for the bind groups 
// This grabs the texture from the Local uniform
@group(1) @binding(1)
var t_diffuse: texture_2d<f32>;
// This grabs the sampler from the Global uniform
@group(0)@binding(2)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // We use the special function `textureSample` to combine the texture data with coords
    let object_color: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    
    // We don't need (or want) much ambient light, so 0.1 is fine
    let ambient_strength = 0.1;
    let ambient_color = light.color * ambient_strength;

    let light_dir = normalize(light.position - in.world_position);
    let view_dir = normalize(globals.view_pos.xyz - in.world_position);
    let half_dir = normalize(view_dir + light_dir);

    let diffuse_strength = max(dot(in.world_normal, light_dir), 0.0);
    let diffuse_color = light.color * diffuse_strength;

    let specular_strength = pow(max(dot(in.world_normal, half_dir), 0.0), 32.0);
    let specular_color = specular_strength * light.color;

    let result = (ambient_color + diffuse_color + specular_color) * object_color.xyz;

    return vec4<f32>(result, object_color.a);
    // return locals.color * vec4<f32>(result, object_color.a);
    // return vec4<f32>(0.0,0.2,0.8, 1.0);
}