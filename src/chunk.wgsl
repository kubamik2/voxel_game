struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) face: u32,
    @location(1) uv: vec2f,
    @location(2) greedy_tiling: vec2f,
    @location(3) base_uv_coords: vec2f,
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
fn vs_main(in: VertexInput, @builtin(vertex_index) vertex_index: u32) -> VertexOutput {
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

    let greedy_x = ((in.packed_instance_data >> 23u) & 15u);
    let greedy_y = ((in.packed_instance_data >> 27u) & 15u);

    
    let face = (in.packed_instance_data >> 12u) & 7u;

    // face rotating
    switch face {
        case 0u: {
            position.x += 1u;
            position.z = (position.z ^ 1u) & 1u;

            position.z *= (greedy_x + 1u);
            position.y *= (greedy_y + 1u);
        }
        case 1u: {
            position.z *= (greedy_x + 1u);
            position.y *= (greedy_y + 1u);
        }
        case 2u: {
            position = position.zyx;
            position.z += 1u;

            position.x *= (greedy_x + 1u);
            position.y *= (greedy_y + 1u);
        }
        case 3u: {
            position = position.zyx;
            position.x = (position.x ^ 1u) & 1u;

            position.x *= (greedy_x + 1u);
            position.y *= (greedy_y + 1u);
        }
        case 4u: {
            position = position.yxz;
            position.y += 1u;

            position.x *= (greedy_y + 1u);
            position.z *= (greedy_x + 1u);
        }
        case 5u: {
            position = position.yxz;
            position.z = (position.z ^ 1u) & 1u;

            position.x *= (greedy_x + 1u);
            position.z *= (greedy_y + 1u);
        }
        default: {}
    }


    // chunk translation
    let chunk_index = in.packed_vertex_data >> 3u;
    position += instance_position;
    position.y += chunk_index * 16u;

    var position_f32 = vec3f(
        f32(position.x) + chunk_translation.x,
        f32(position.y),
        f32(position.z) + chunk_translation.y,
    );

    // uv creation
    var uv = vec2f();
    switch vertex_index % 4u {
        case 0u: {
            uv = vec2f(0.0, 0.0625);
        }
        case 1u: {
            uv = vec2f(0.0625, 0.0625);
            // position_f32.z += f32(greedy_x);
        }
        case 2u: {
            uv = vec2f(0.0, 0.0);
            // position_f32.y += f32(greedy_y);
        }
        case 3u: {
            uv = vec2f(0.0625, 0.0);
            // position_f32.z += f32(greedy_x);
            // position_f32.y += f32(greedy_y);
        }
        default: {} 
    }

    let texture_index = (in.packed_instance_data >> 15u) & 255u;

    uv.x += f32((texture_index) % 16u) * 0.0625;
    uv.y += f32((texture_index - 1u) / 16u) * 0.0625;

    // view projection
    out.clip_position = camera.view_projection * vec4f(position_f32, 1.0);

    out.face = face;
    out.uv = uv;
    out.greedy_tiling = vec2f(f32(greedy_x + 1u), f32(greedy_y + 1u));
    out.base_uv_coords = vec2f(f32((texture_index) % 16u) * 0.0625, f32((texture_index - 1u) / 16u) * 0.0625);
    return out;
}

@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    var shade = vec4f(1.0, 1.0, 1.0, 1.0);
    switch in.face {
        case 1u: {
            shade = vec4f(0.3, 0.3, 0.3, 1.0);
        }
        case 3u: {
            shade = vec4f(0.3, 0.3, 0.3, 1.0);
        }
        case 4u: {
            shade = vec4f(1.3, 1.3, 1.3, 1.0);
        }
        default: {}
    }
    var uv = in.uv;
    let local_uv_coords = uv - in.base_uv_coords;

    uv.x = in.base_uv_coords.x + (local_uv_coords.x * in.greedy_tiling.x) % 0.0625;
    uv.y = in.base_uv_coords.y + (local_uv_coords.y * in.greedy_tiling.y) % 0.0625;

    return textureSample(t_diffuse, s_diffuse, uv) * shade;

}