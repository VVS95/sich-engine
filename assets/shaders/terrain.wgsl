#import bevy_pbr::{
    mesh_functions,
    view_transformations::position_world_to_clip,
}

struct VertexInput {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) splat_uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv_0: vec2<f32>,
    @location(1) uv_1: vec2<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var atlas: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var atlas_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var indices: texture_2d<u32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var weights: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(5) var weights_sampler: sampler;

@vertex
fn vertex(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let world_from_local = mesh_functions::get_world_from_local(input.instance_index);
    let world_position = mesh_functions::mesh_position_local_to_world(
        world_from_local,
        vec4<f32>(input.position, 1.0),
    );

    output.position = position_world_to_clip(world_position.xyz);
    output.uv_0 = input.uv;
    output.uv_1 = input.splat_uv;
    return output;
}

fn atlas_uv(tile_id: u32, local_uv: vec2<f32>) -> vec2<f32> {
    let columns = 4u;
    let rows = 37.0;
    let tile = vec2<f32>(
        f32(tile_id % columns),
        f32(tile_id / columns),
    );
    let pad = 0.015; 
    let padded_uv = clamp(local_uv, vec2<f32>(pad), vec2<f32>(1.0 - pad));
    return (tile + padded_uv) / vec2<f32>(f32(columns), rows);
}

// НОВА ФУНКЦІЯ: Рахує фінальний колір для одного конкретного пікселя сплетмапи
fn sample_splat_pixel(
    coord: vec2<i32>, 
    dim: vec2<i32>, 
    local_uv: vec2<f32>
) -> vec4<f32> {
    let safe_coord = clamp(coord, vec2<i32>(0), dim - vec2<i32>(1));
    
    // Жорстко читаємо і індекси, і ваги! Жодного розсинхрону.
    let idx = textureLoad(indices, safe_coord, 0);
    let wgt = textureLoad(weights, safe_coord, 0);
    
    let c0 = textureSampleLevel(atlas, atlas_sampler, atlas_uv(idx.x, local_uv), 0.0);
    let c1 = textureSampleLevel(atlas, atlas_sampler, atlas_uv(idx.y, local_uv), 0.0);
    let c2 = textureSampleLevel(atlas, atlas_sampler, atlas_uv(idx.z, local_uv), 0.0);
    let c3 = textureSampleLevel(atlas, atlas_sampler, atlas_uv(idx.w, local_uv), 0.0);
    
    return c0 * wgt.x + c1 * wgt.y + c2 * wgt.z + c3 * wgt.w;
}

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    let dim = vec2<i32>(textureDimensions(indices));
    let f_dim = vec2<f32>(dim);
    
    // Вираховуємо точні координати для нашої ручної фільтрації
    let pixel_pos = input.uv_1 * f_dim - vec2<f32>(0.5);
    let base_coord = vec2<i32>(floor(pixel_pos));
    let fract_pos = fract(pixel_pos);
    
    // Отримуємо 4 готові кольори з 4 сусідніх пікселів сплетмапи
    let color_00 = sample_splat_pixel(base_coord + vec2<i32>(0, 0), dim, input.uv_0);
    let color_10 = sample_splat_pixel(base_coord + vec2<i32>(1, 0), dim, input.uv_0);
    let color_01 = sample_splat_pixel(base_coord + vec2<i32>(0, 1), dim, input.uv_0);
    let color_11 = sample_splat_pixel(base_coord + vec2<i32>(1, 1), dim, input.uv_0);
    
    // Математично змішуємо кольори (Білінійна інтерполяція)
    let color_y0 = mix(color_00, color_10, fract_pos.x);
    let color_y1 = mix(color_01, color_11, fract_pos.x);
    return mix(color_y0, color_y1, fract_pos.y);
}