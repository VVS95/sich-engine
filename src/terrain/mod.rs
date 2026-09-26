pub mod grid;
pub mod material; // Підключаємо твій новий файл
pub mod splatmap;

use bevy::prelude::*;
use grid::TerrainGrid;
use material::{TerrainMaterialPlugin, TerrainSplatMaterial};
use splatmap::{SPLATMAP_RESOLUTION, TerrainSplatmap};

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        // Додаємо плагін матеріалу
        app.add_plugins(TerrainMaterialPlugin);
        app.add_systems(Startup, spawn_terrain);
    }
}

fn spawn_terrain(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    // Змінюємо StandardMaterial на наш кастомний!
    mut materials: ResMut<Assets<TerrainSplatMaterial>>,
) {
    let texture_handle: Handle<Image> = asset_server.load("gsc://TILES3.BMP");
    let splatmap = TerrainSplatmap::new(&mut images, UVec2::splat(SPLATMAP_RESOLUTION));
    let splatmap_handle = splatmap.handle.clone();
    commands.insert_resource(splatmap);
    let grid = TerrainGrid::new(20, 20, 5);

    let mesh = grid.generate_mesh();

    // Створюємо наш крутий матеріал
    let material = TerrainSplatMaterial {
        atlas_texture: texture_handle,
        splatmap_texture: splatmap_handle,
        atlas_size: Vec2::new(4.0, 37.0), // Передаємо розміри атласу
        tile_indices: UVec4::new(5, 20, 0, 0),
    };

    let offset = -((20.0 * grid.tile_size) / 2.0);

    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(material)),
        grid,
        Transform::from_xyz(offset, 0.0, offset),
    ));
}
