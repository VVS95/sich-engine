use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

pub const SPLATMAP_RESOLUTION: u32 = 256;

#[derive(Resource, Clone)]
pub struct TerrainSplatmap {
    pub handle: Handle<Image>,
    pub resolution: UVec2,
}

impl TerrainSplatmap {
    pub fn new(images: &mut Assets<Image>, resolution: UVec2) -> Self {
        let pixel_count = (resolution.x * resolution.y) as usize;
        let mut data = vec![0u8; pixel_count * 4];
        for pixel in data.chunks_exact_mut(4) {
            pixel[0] = u8::MAX;
        }

        let mut image = Image::new(
            Extent3d {
                width: resolution.x,
                height: resolution.y,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            TextureFormat::Rgba8Unorm,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        );
        image.sampler = ImageSampler::linear();

        Self {
            handle: images.add(image),
            resolution,
        }
    }
}
