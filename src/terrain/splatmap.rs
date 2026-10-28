use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

#[derive(Resource, Clone)]
pub struct TerrainSplatmap {
    /// Legacy alias for the weights image until brush painting is updated.
    pub handle: Handle<Image>,
    pub indices: Handle<Image>,
    pub weights: Handle<Image>,
    pub resolution: UVec2,
}

impl TerrainSplatmap {
    pub fn new(images: &mut Assets<Image>, resolution: UVec2) -> Self {
        let pixel_count = (resolution.x * resolution.y) as usize;
        let extent = Extent3d {
            width: resolution.x,
            height: resolution.y,
            depth_or_array_layers: 1,
        };
        let usages = RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD;

        let indices_data = vec![0u8; pixel_count * 4];
        let mut indices_image = Image::new(
            extent,
            TextureDimension::D2,
            indices_data,
            TextureFormat::Rgba8Uint,
            usages,
        );
        indices_image.sampler = ImageSampler::nearest();

        let mut weights_data = vec![0u8; pixel_count * 4];
        for pixel in weights_data.chunks_exact_mut(4) {
            pixel[0] = u8::MAX;
        }
        let mut weights_image = Image::new(
            extent,
            TextureDimension::D2,
            weights_data,
            TextureFormat::Rgba8Unorm,
            usages,
        );
        weights_image.sampler = ImageSampler::linear();

        let indices = images.add(indices_image);
        let weights = images.add(weights_image);

        Self {
            handle: weights.clone(),
            indices,
            weights,
            resolution,
        }
    }
}
