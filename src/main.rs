mod assets;
mod camera;
mod interaction;
mod terrain;

use assets::GscAssetsPlugin;
use bevy::asset::{
    AssetApp, AssetPlugin,
    io::{AssetSourceBuilder, AssetSourceId, file::FileAssetReader},
};
use bevy::prelude::*;
use camera::CameraPlugin;
use interaction::InteractionPlugin;
use std::path::PathBuf;

fn main() {
    App::new()
        .register_asset_source(
            AssetSourceId::Name("gsc".into()),
            AssetSourceBuilder::new(move || {
                Box::new(FileAssetReader::new(PathBuf::from(
                    "C:/GSCExtractor/GSC File Utility/extracted",
                )))
            }),
        )
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin {
                    file_path: "./assets".to_string(),
                    ..default()
                }),
        )
        .add_plugins(CameraPlugin)
        .add_plugins(GscAssetsPlugin)
        .add_plugins(terrain::TerrainPlugin)
        .add_plugins(InteractionPlugin)
        .run();
}
