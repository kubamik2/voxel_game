struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec3f,
    @location(1) face: u32,

}

struct VertexInput {
    @location(0) packed_vertex_data: u32,
}

struct CameraUniform {
    view_projection: mat4x4<f32>
}

struct InstanceInput {
    @location(2) matrix_0: vec4f,
    @location(3) matrix_1: vec4f,
    @location(4) matrix_2: vec4f,
    @location(5) matrix_3: vec4f,
    @location(6) texture_offset: vec2f
}

@group(1) @binding(0) var<uniform> camera: CameraUniform;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let position = vec3f(
        f32(in.packed_vertex_data & 63u),
        f32(in.packed_vertex_data >> 6u & 63u),
        f32(in.packed_vertex_data >> 12u & 63u)
    );

    let face = in.packed_vertex_data >> 18u;

    out.clip_position = camera.view_projection * vec4f(position, 1.0);

    out.position = position;
    out.face = face;
    return out;
}

@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;

@group(2) @binding(0) var t_mat: texture_3d<u32>;
@group(2) @binding(1) var s_mat: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    var mat_tex_coords = vec3u(
        u32(floor(in.position.x)),
        u32(floor(in.position.z)),
        u32(floor(in.position.y)),
    );

    let tex_x = in.position.x - floor(in.position.x);
    let tex_y = (1.0 - (in.position.y - floor(in.position.y))) * 0.0625;
    let tex_z = in.position.z - floor(in.position.z);

    var tex_coords = vec2f();

    switch in.face {
        case 0u: {
            mat_tex_coords.x -= 1u;
            tex_coords = vec2f((1.0 - tex_z) * 0.0625, tex_y);
        }
        case 1u: {
            tex_coords = vec2f(tex_z * 0.0625, tex_y);
        }
        case 2u: {
            mat_tex_coords.y -= 1u; // positive sides edge case
            tex_coords = vec2f(tex_x * 0.0625, tex_y);
        }
        case 3u: {
            tex_coords = vec2f((1.0 - tex_x) * 0.0625, tex_y);
        }
        case 4u: {
            mat_tex_coords.z -= 1u;
            tex_coords = vec2f(tex_x * 0.0625, tex_z * 0.0625);
        }
        case 5u: {
            tex_coords = vec2f(tex_x * 0.0625, tex_z * 0.0625);
        }
        default: {}
    }

    let material = textureLoad(t_mat, mat_tex_coords, 0i).x;
    tex_coords.x += (f32(material) - 1.0) * 0.0625;

    return textureSample(t_diffuse, s_diffuse, tex_coords);
}