use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::asset::RenderAssetUsages;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_test_tile);
    }
}

fn spawn_test_tile(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Loading the texture from the asset server. Make sure the path is correct and the file exists.
    let texture_handle: Handle<Image> = asset_server.load("TILES3.GBMP");

    // Creating a mesh for a single tile.
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    // Square vertices coordinates (X, Y, Z). Size 64x64.
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![
        [0.0, 0.0, 0.0],
        [64.0, 0.0, 0.0],
        [64.0, 0.0, 64.0],
        [0.0, 0.0, 64.0],
    ]);

    // Normals (where the surface "faces" — upwards along the Y-axis)
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
    ]);

    // UV mapping (how to apply the texture).
    // For now, we're applying the entire TILES3 image to a single square for testing.
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
    ]);

    // Indices (joining 4 points into 2 triangles)
    mesh.insert_indices(Indices::U32(vec![0, 2, 1, 0, 3, 2]));

    // Creating a material for the mesh, using the loaded texture.
    let material = StandardMaterial {
        base_color_texture: Some(texture_handle),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    };

    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(material)),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}