#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}

@group(2) @binding(0) var atlas_texture: texture_2d<f32>;
@group(2) @binding(1) var atlas_sampler: sampler;
@group(2) @binding(2) var<uniform> atlas_size: vec2<f32>;
@group(2) @binding(3) var splatmap_texture: texture_2d<f32>;
@group(2) @binding(4) var splatmap_sampler: sampler;
@group(2) @binding(5) var<uniform> tile_indices: vec4<u32>;

struct VertexInput {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>, // Локальні координати квадрата (0.0 .. 1.0)
    @location(3) splat_uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) splat_uv: vec2<f32>,
}

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    var world_from_local = get_world_from_local(in.instance_index);
    out.clip_position = mesh_position_local_to_clip(world_from_local, vec4<f32>(in.position, 1.0));
    out.uv = in.uv;
    out.splat_uv = in.splat_uv;
    return out;
}

// Функція для вирізання правильного тайлу з TILES3.BMP
fn get_atlas_uv(local_uv: vec2<f32>, tile_index: u32) -> vec2<f32> {
    let cols = u32(atlas_size.x);
    let tx = f32(tile_index % cols);
    let ty = f32(tile_index / cols);

    let u_step = 1.0 / atlas_size.x;
    let v_step = 1.0 / atlas_size.y;

    let u0 = tx * u_step;
    // Віднімаємо від 1.0, щоб читати BMP згори вниз (як ми це робили в Rust)
    let v1 = 1.0 - (ty * v_step);
    let v0 = v1 - v_step;

    return vec2<f32>(
        u0 + local_uv.x * u_step,
        v0 + local_uv.y * v_step
    );
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // 1. Вираховуємо координати в атласі для всіх 4 потенційних текстур на цій точці
    let weights = textureSample(splatmap_texture, splatmap_sampler, in.splat_uv);
    let uv0 = get_atlas_uv(in.uv, tile_indices.x);
    let uv1 = get_atlas_uv(in.uv, tile_indices.y);
    let uv2 = get_atlas_uv(in.uv, tile_indices.z);
    let uv3 = get_atlas_uv(in.uv, tile_indices.w);

    // 2. Беремо кольори з цих текстур
    let color0 = textureSample(atlas_texture, atlas_sampler, uv0);
    let color1 = textureSample(atlas_texture, atlas_sampler, uv1);
    let color2 = textureSample(atlas_texture, atlas_sampler, uv2);
    let color3 = textureSample(atlas_texture, atlas_sampler, uv3);

    // 3. Змішуємо їх за вагою (сплаттінг)
    let final_color = color0 * weights.x +
                      color1 * weights.y +
                      color2 * weights.z +
                      color3 * weights.w;

    return final_color;
}