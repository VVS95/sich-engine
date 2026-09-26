use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::render::mesh::MeshVertexBufferLayoutRef;
use bevy::render::render_resource::{RenderPipelineDescriptor, SpecializedMeshPipelineError};
use bevy::prelude::*;
use bevy::render::mesh::MeshVertexAttribute;
use bevy::render::render_resource::{AsBindGroup, VertexFormat};
use bevy::shader::ShaderRef;

// --- CUSTOM VERTEX ATTRIBUTES ---

/// Stores up to 4 texture IDs from the atlas for a single vertex.
/// E.g., [GrassID, SandID, DirtID, 0]
pub const ATTRIBUTE_BLEND_INDICES: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertex_Blend_Indices", 3, VertexFormat::Uint32x4);

/// Stores the blending weights (0.0 to 1.0) for the 4 textures at this vertex.
/// E.g., [0.5, 0.5, 0.0, 0.0] means 50% Grass, 50% Sand.
pub const ATTRIBUTE_BLEND_WEIGHTS: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertex_Blend_Weights", 4, VertexFormat::Float32x4);

// --- MATERIAL STRUCTURE ---

/// Custom material for rendering smooth terrain splatting using a texture atlas.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct TerrainSplatMaterial {
    /// The main texture atlas containing all terrain tiles.
    #[texture(0)]
    #[sampler(1)]
    pub atlas_texture: Handle<Image>,

    /// Stores atlas grid dimensions (X = columns, Y = rows) to calculate UVs inside the shader.
    #[uniform(2)]
    pub atlas_size: Vec2,
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
            ATTRIBUTE_BLEND_INDICES.at_shader_location(3),
            ATTRIBUTE_BLEND_WEIGHTS.at_shader_location(4),
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