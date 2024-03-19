struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec3f,
    @location(1) face: u32,

}

struct VertexInput {
    @location(0) packed_vertex_data: u32,
    @location(1) chunk_translation: vec3i,
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

    out.clip_position = camera.view_projection * vec4f(position.x + f32(in.chunk_translation.x), position.y + f32(in.chunk_translation.y), position.z + f32(in.chunk_translation.z), 1.0);

    out.position = position;
    out.face = face;
    return out;
}

@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;

@group(2) @binding(0) var t_mat: texture_3d<u32>;

@group(3) @binding(0) var f_mat: texture_3d<u32>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    var mat_tex_coords = vec3u(
        u32(in.position.x + 0.00001), // fixes visual glitches on some GPUs, maybe because of float inprecision?
        u32(in.position.z + 0.00001),
        u32(in.position.y + 0.00001),
    );

    let tex_x = fract(in.position.x) * 0.0625;
    let tex_y = (1.0 - fract(in.position.y)) * 0.0625;
    let tex_z = fract(in.position.z) * 0.0625;

    var tex_coords = vec2f();

    switch in.face {
        case 0u: {
            tex_coords = vec2f(0.0625 - tex_z, tex_y);
        }
        case 1u: {
            tex_coords = vec2f(tex_z, tex_y);
        }
        case 2u: {
            tex_coords = vec2f(tex_x, tex_y);
        }
        case 3u: {
            tex_coords = vec2f(0.0625 - tex_x, tex_y);
        }
        case 4u: {
            tex_coords = vec2f(tex_x, tex_z);
        }
        case 5u: {
            tex_coords = vec2f(tex_x, tex_z);
        }
        default: {}
    }
    mat_tex_coords -= vec3u(u32(in.face == 0u), u32(in.face == 2u), u32(in.face == 4u));
    
    let face_visibility_bitmask = textureLoad(f_mat, mat_tex_coords, 0i).x;
    if (1u << in.face & face_visibility_bitmask) == 0u { discard; }

    let material = textureLoad(t_mat, mat_tex_coords, 0i).x;
    if material == 0u { discard; }
    tex_coords.x += (f32(material)) * 0.0625;

    return textureSample(t_diffuse, s_diffuse, tex_coords);
    // return vec4f(0.0, 0.0, 0.0, 1.0);
}