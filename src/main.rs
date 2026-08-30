mod camera;
mod terrain;
mod assets;

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
                    resolution: (1280.0_f32, 720.0_f32).into(),
                    ..default()
                }),
                ..default()
            })
        )
        .add_plugins(CameraPlugin)
        .add_plugins(GscAssetsPlugin)
        .run();
}