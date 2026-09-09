mod camera;
mod assets;
mod terrain;


use bevy::prelude::*;
use bevy::asset::AssetPlugin;
use camera::CameraPlugin;
use assets::GscAssetsPlugin;
fn main() {
    App::new()
        .add_plugins(DefaultPlugins
            .set(AssetPlugin {
                file_path: "C:/GSCExtractor/GSC File Utility/extracted".to_string(),
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Sich Engine".to_string(),
                    resolution: (1280, 720).into(),
                    ..default()
                }),
                ..default()
            })
        )
        .add_plugins(CameraPlugin)
        .add_plugins(GscAssetsPlugin)
        .add_plugins(terrain::TerrainPlugin)
        .run();
}