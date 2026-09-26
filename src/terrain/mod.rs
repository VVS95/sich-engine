pub mod grid;
pub mod material; // Підключаємо твій новий файл

use bevy::prelude::*;
use grid::TerrainGrid;
use material::{TerrainSplatMaterial, TerrainMaterialPlugin};

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
    // Змінюємо StandardMaterial на наш кастомний!
    mut materials: ResMut<Assets<TerrainSplatMaterial>>,
) {
    let texture_handle: Handle<Image> = asset_server.load("gsc://TILES3.BMP");
    let mut grid = TerrainGrid::new(20, 20, 5);
    
    // Малюємо тестову дорогу
    for i in 0..20 {
        let index = grid.get_index(i, i);
        grid.tiles[index] = 20; 
        if i < 19 {
            let index_right = grid.get_index(i + 1, i);
            grid.tiles[index_right] = 20;
        }
    }

    let mesh = grid.generate_mesh();

    // Створюємо наш крутий матеріал
    let material = TerrainSplatMaterial {
        atlas_texture: texture_handle,
        atlas_size: Vec2::new(4.0, 37.0), // Передаємо розміри атласу
    };

    let offset = -((20.0 * grid.tile_size) / 2.0);

    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(material)),
        grid,
        Transform::from_xyz(offset, 0.0, offset),
    ));
}