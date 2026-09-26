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
    @location(0) uv: vec2<f32>,
    @location(1) splat_uv: vec2<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var atlas_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var atlas_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var<uniform> atlas_size: vec2<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var splatmap_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var splatmap_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(5) var<uniform> tile_indices: vec4<u32>;

@vertex
fn vertex(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let world_from_local = mesh_functions::get_world_from_local(input.instance_index);
    let world_position = mesh_functions::mesh_position_local_to_world(
        world_from_local,
        vec4<f32>(input.position, 1.0),
    );

    output.position = position_world_to_clip(world_position.xyz);
    output.uv = input.uv;
    output.splat_uv = input.splat_uv;
    return output;
}

fn atlas_uv(tile_index: u32, local_uv: vec2<f32>) -> vec2<f32> {
    let columns = u32(atlas_size.x);
    let tile = vec2<f32>(
        f32(tile_index % columns),
        f32(tile_index / columns),
    );
    return (tile + local_uv) / atlas_size;
}

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    let weights = textureSample(splatmap_texture, splatmap_sampler, input.splat_uv);
    let color_0 = textureSample(
        atlas_texture,
        atlas_sampler,
        atlas_uv(tile_indices.x, input.uv),
    );
    let color_1 = textureSample(
        atlas_texture,
        atlas_sampler,
        atlas_uv(tile_indices.y, input.uv),
    );
    let color_2 = textureSample(
        atlas_texture,
        atlas_sampler,
        atlas_uv(tile_indices.z, input.uv),
    );
    let color_3 = textureSample(
        atlas_texture,
        atlas_sampler,
        atlas_uv(tile_indices.w, input.uv),
    );

    return color_0 * weights.x
        + color_1 * weights.y
        + color_2 * weights.z
        + color_3 * weights.w;
}
