#import bevy_pbr::{
    mesh_functions,
    view_transformations::position_world_to_clip,
}

struct VertexInput {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) blend_indices: vec4<u32>,
    @location(4) blend_weights: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) blend_indices: vec4<u32>,
    @location(2) blend_weights: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var atlas_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var atlas_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var<uniform> atlas_size: vec2<f32>;

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
    output.blend_indices = input.blend_indices;
    output.blend_weights = input.blend_weights;
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
    let color_0 = textureSample(
        atlas_texture,
        atlas_sampler,
        atlas_uv(input.blend_indices.x, input.uv),
    );
    let color_1 = textureSample(
        atlas_texture,
        atlas_sampler,
        atlas_uv(input.blend_indices.y, input.uv),
    );
    let color_2 = textureSample(
        atlas_texture,
        atlas_sampler,
        atlas_uv(input.blend_indices.z, input.uv),
    );
    let color_3 = textureSample(
        atlas_texture,
        atlas_sampler,
        atlas_uv(input.blend_indices.w, input.uv),
    );

    return color_0 * input.blend_weights.x
        + color_1 * input.blend_weights.y
        + color_2 * input.blend_weights.z
        + color_3 * input.blend_weights.w;
}
