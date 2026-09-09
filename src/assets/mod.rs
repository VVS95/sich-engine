use bevy::prelude::*;
use bevy::asset::{AssetLoader, LoadContext, io::Reader, AsyncReadExt, ReadAssetBytesError};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::asset::RenderAssetUsages;
use thiserror::Error;

pub struct GscAssetsPlugin;

impl Plugin for GscAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_loader(GscBmpLoader);
    }
}

#[derive(Debug, Error)]
pub enum GscBmpLoaderError {
    #[error("Could not read asset: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not read palette bytes: {0}")]
    ReadBytes(#[from] ReadAssetBytesError),
    #[error("Invalid BMP format")]
    InvalidFormat,
}

#[derive(Default, bevy::reflect::TypePath)]
pub struct GscBmpLoader;

impl AssetLoader for GscBmpLoader {
    type Asset = Image;
    type Settings = ();
    type Error = GscBmpLoaderError;

    fn extensions(&self) -> &[&str] {
        &["gbmp", "GBMP"]
    }

    #[allow(refining_impl_trait)]
    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> { 
        
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let palette_path = "AGEW_1.PAL";
        
        let palette_bytes = load_context.read_asset_bytes(palette_path).await?;

        let mut palette = Vec::with_capacity(256);
        for i in 0..256 {
            let base = i * 3;
            if base + 2 < palette_bytes.len() {
                palette.push([palette_bytes[base], palette_bytes[base + 1], palette_bytes[base + 2]]);
            }
        }

        if bytes.len() < 54 {
            return Err(GscBmpLoaderError::InvalidFormat);
        }

        let data_offset = u32::from_le_bytes([bytes[10], bytes[11], bytes[12], bytes[13]]) as usize;
        let width = i32::from_le_bytes([bytes[18], bytes[19], bytes[20], bytes[21]]) as usize;
        let height = i32::from_le_bytes([bytes[22], bytes[23], bytes[24], bytes[25]]).abs() as usize;

        let mut rgba_buf = vec![0u8; width * height * 4];
        let row_stride = (width + 3) & !3;

        for y in 0..height {
            let src_row = height - 1 - y;
            let row_start = data_offset + (src_row * row_stride);

            for x in 0..width {
                if row_start + x < bytes.len() {
                    let idx = bytes[row_start + x];
                    let p = &palette[idx as usize];

                    let dst_idx = (y * width + x) * 4;
                    rgba_buf[dst_idx]     = p[0]; // R
                    rgba_buf[dst_idx + 1] = p[1]; // G
                    rgba_buf[dst_idx + 2] = p[2]; // B
                    
                    rgba_buf[dst_idx + 3] = if idx == 0 { 0 } else { 255 }; 
                }
            }
        }

        let image = Image::new(
            Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            rgba_buf,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        );

        Ok(image)
    }
}