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

fn fragment_noise(position: vec2<f32>) -> f32 {
    let value = sin(dot(position, vec2<f32>(12.9898, 78.233))) * 43758.5453;
    return fract(value);
}

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    let dim = vec2<f32>(textureDimensions(indices));
    let tex_coord = vec2<i32>(clamp(input.uv_1 * dim, vec2<f32>(0.0), dim - vec2<f32>(1.0)));
    
    let sampled_indices = textureLoad(indices, tex_coord, 0);
    let sampled_weights = textureSample(weights, weights_sampler, input.uv_1);
    
    let noise = fragment_noise(input.uv_1 * 2000.0);
    
    let cumulative_0 = sampled_weights.x;
    let cumulative_1 = cumulative_0 + sampled_weights.y;
    let cumulative_2 = cumulative_1 + sampled_weights.z;

    var tile_id = sampled_indices.w; 
    
    if noise < cumulative_0 {
        tile_id = sampled_indices.x; 
    } else if noise < cumulative_1 {
        tile_id = sampled_indices.y; 
    } else if noise < cumulative_2 {
        tile_id = sampled_indices.z; 
    }

    let final_uv = atlas_uv(tile_id, input.uv_0);
    return textureSampleLevel(atlas, atlas_sampler, final_uv, 0.0);
}