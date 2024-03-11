struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2f
}

struct VertexInput {
    @location(0) position: vec3f,
    @location(1) tex_coords: vec2f
}

struct CameraUniform {
    view_projection: mat4x4<f32>
}

struct InstanceInput {
    @location(2) matrix_0: vec4f,
    @location(3) matrix_1: vec4f,
    @location(4) matrix_2: vec4f,
    @location(5) matrix_3: vec4f,
}

@group(1) @binding(0) var<uniform> camera: CameraUniform;

@vertex
fn vs_main(in: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;
    let model_matrix = mat4x4f(
        instance.matrix_0,
        instance.matrix_1,
        instance.matrix_2,
        instance.matrix_3
    );
    let clip_position = vec4f(in.position, 1.0);
    out.clip_position = camera.view_projection * model_matrix * vec4f(in.position, 1.0);
    out.tex_coords = in.tex_coords;
    return out;
}

@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}