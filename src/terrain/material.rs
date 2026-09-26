use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::mesh::MeshVertexBufferLayoutRef;
use bevy::render::render_resource::AsBindGroup;
use bevy::render::render_resource::{RenderPipelineDescriptor, SpecializedMeshPipelineError};
use bevy::shader::ShaderRef;

// --- MATERIAL STRUCTURE ---

/// Custom material for rendering smooth terrain splatting using a texture atlas.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct TerrainSplatMaterial {
    /// The main texture atlas containing all terrain tiles.
    #[texture(0)]
    #[sampler(1)]
    pub atlas_texture: Handle<Image>,

    #[texture(3)]
    #[sampler(4)]
    pub splatmap_texture: Handle<Image>,

    /// Stores atlas grid dimensions (X = columns, Y = rows) to calculate UVs inside the shader.
    #[uniform(2)]
    pub atlas_size: Vec2,

    #[uniform(5)]
    pub tile_indices: UVec4,
}

// --- TRAIT IMPLEMENTATION ---

impl Material for TerrainSplatMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/terrain.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/terrain.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            Mesh::ATTRIBUTE_UV_1.at_shader_location(3),
        ])?;

        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}
// --- PLUGIN ---

/// Plugin responsible for registering the custom terrain material in the Bevy app.
pub struct TerrainMaterialPlugin;

impl Plugin for TerrainMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<TerrainSplatMaterial>::default());
    }
}
