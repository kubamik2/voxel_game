struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec3f,
    @location(1) face: u32,
    @location(2) texture_index: u32,
    @location(3) chunk_index: u32,
}

struct VertexInput {
    @location(0) packed_vertex_data: u32,
    @location(1) packed_instance_data: u32,
}

struct CameraUniform {
    view_projection: mat4x4<f32>
}

@group(1) @binding(0) var<uniform> camera: CameraUniform;
@group(2) @binding(0) var<uniform> chunk_translation: vec2f;

@vertex
fn vs_main(in: VertexInput, @builtin(instance_index) i: u32) -> VertexOutput {
    var out: VertexOutput;

    let instance_position = vec3u(
        in.packed_instance_data & 15u,
        (in.packed_instance_data >> 4u) & 15u,
        (in.packed_instance_data >> 8u) & 15u,
    );

    var position = vec3u(
        (in.packed_vertex_data & 1u),
        ((in.packed_vertex_data >> 1u) & 1u),
        ((in.packed_vertex_data >> 2u) & 1u),
    );

    let chunk_index = in.packed_vertex_data >> 3u;

    let face = (in.packed_instance_data >> 12u) & 7u;
    switch face {
        case 0u: {
            position.x += 1u;
            position.z = (position.z ^ 1u) & 1u;
        }
        case 2u: {
            position = position.zyx;
            position.z += 1u;
        }
        case 3u: {
            position = position.zyx;
            position.x = (position.x ^ 1u) & 1u;
        }
        case 4u: {
            position = position.yxz;
            position.y += 1u;
        }
        case 5u: {
            position = position.yxz;
            position.z = (position.z ^ 1u) & 1u;
        }
        default: {}
    }
    position += instance_position;
    position.y += chunk_index * 16u;

    let position_f32 = vec3f(
        f32(position.x) + chunk_translation.x,
        f32(position.y),
        f32(position.z) + chunk_translation.y,
    );

    let texture_index = in.packed_instance_data >> 15u;

    out.clip_position = camera.view_projection * vec4f(position_f32.x, position_f32.y, position_f32.z, 1.0);

    out.position = position_f32;
    out.face = face;
    out.texture_index = texture_index;
    out.chunk_index = chunk_index;
    return out;
}

@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    let uv3 = vec3f(
        fract(in.position.x) * 0.0625,
        (1.0 - fract(in.position.y)) * 0.0625,
        fract(in.position.z) * 0.0625,
    );

    var uv2 = vec2f();
    switch in.face {
        case 0u: {
            uv2 = uv3.zy;
        }
        case 1u: {
            uv2 = uv3.zy;
        }
        case 2u: {
            uv2 = uv3.xy;
        }
        case 3u: {
            uv2 = uv3.xy;
        }
        case 4u: {
            uv2 = uv3.xz;
        }
        case 5u: {
            uv2 = uv3.xz;
        }
        default: {}
    }

    uv2.x += f32(in.texture_index - 1u) * 0.0625;

    return textureSample(t_diffuse, s_diffuse, uv2);

//     let v = f32(in.chunk_index) / 23.0;
//     return vec4f(v, v, v, 1.0);
}