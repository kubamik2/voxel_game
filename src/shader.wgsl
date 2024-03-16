struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2f,
    @location(1) tile: vec2f
}

struct VertexInput {
    @location(0) position: vec3f,
    @location(1) packed_vertex_data: u32,
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
    let clip_position = vec4f(in.position, 1.0);
    out.clip_position = camera.view_projection * vec4f(in.position, 1.0);

    let tex_coords = vec2f(
        f32(in.packed_vertex_data >> u32(24)) / 256.0,
        f32(in.packed_vertex_data >> u32(16) & u32(255)) / 256.0
    );

    let tile = vec2f(
        f32(in.packed_vertex_data >> u32(8) & u32(255)),
        f32(in.packed_vertex_data & u32(255))
    );
    
    out.tex_coords = tex_coords;
    out.tile = tile;
    return out;
}

@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    if in.tile.x == 1.0 && in.tile.y == 1.0 {
        return textureSample(t_diffuse, s_diffuse, in.tex_coords);
    }
    
    var tex_coords = vec2f();
    let base_tile_coords = floor(in.tex_coords / 0.0625) * 0.0625;
    let local_tile_coords = in.tex_coords - base_tile_coords;


    tex_coords.x = base_tile_coords.x + (local_tile_coords.x * in.tile.x) % 0.0625;
    tex_coords.y = base_tile_coords.y + (local_tile_coords.y * in.tile.y) % 0.0625;
    return textureSample(t_diffuse, s_diffuse, tex_coords);
}