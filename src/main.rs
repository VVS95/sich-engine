mod camera;
mod assets;
mod terrain;
mod interaction;

use bevy::prelude::*;
use camera::CameraPlugin;
use assets::GscAssetsPlugin;
use interaction::InteractionPlugin;
use bevy::asset::{
    io::{file::FileAssetReader, AssetSourceBuilder, AssetSourceId},
    AssetApp, AssetPlugin,
};
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