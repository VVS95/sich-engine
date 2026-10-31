pub mod grid;
pub mod map_io;
pub mod material; // Підключаємо твій новий файл
pub mod splatmap;

use bevy::prelude::*;
use grid::TerrainGrid;
use material::{TerrainMaterial, TerrainMaterialPlugin};
use splatmap::TerrainSplatmap;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        // Додаємо плагін матеріалу
        app.add_plugins(TerrainMaterialPlugin);
        app.init_resource::<map_io::MapIoState>()
            .add_message::<map_io::SaveMapRequest>()
            .add_message::<map_io::LoadMapRequest>()
            .add_systems(Startup, spawn_terrain)
            .add_systems(Update, (map_io::save_map_system, map_io::load_map_system));
    }
}

fn spawn_terrain(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    // Змінюємо StandardMaterial на наш кастомний!
    mut materials: ResMut<Assets<TerrainMaterial>>,
) {
    let map_size = 256;
    let texture_handle: Handle<Image> = asset_server.load("gsc://TILES3.BMP");
    let splatmap = TerrainSplatmap::new(&mut images, UVec2::splat(map_size as u32));
    let indices_handle = splatmap.indices.clone();
    let weights_handle = splatmap.weights.clone();
    commands.insert_resource(splatmap);
    let grid = TerrainGrid::new(map_size, map_size, 5);

    let mesh = grid.generate_mesh();

    // Створюємо наш крутий матеріал
    let material = TerrainMaterial {
        atlas: texture_handle,
        indices: indices_handle,
        weights: weights_handle,
    };

    let offset = -(grid.width as f32 * grid.tile_size) / 2.0;

    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(material)),
        grid,
        Transform::from_xyz(offset, 0.0, offset),
    ));
}
