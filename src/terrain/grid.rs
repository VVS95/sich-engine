use crate::terrain::material::{ATTRIBUTE_BLEND_INDICES, ATTRIBUTE_BLEND_WEIGHTS};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::asset::RenderAssetUsages;

#[derive(Component)]
pub struct TerrainGrid {
    pub width: usize,
    pub height: usize,
    pub tile_size: f32,
    pub atlas_cols: usize,
    pub atlas_rows: usize,
    pub tiles: Vec<usize>,
}

impl TerrainGrid {
    pub fn new(width: usize, height: usize, default_tile_id: usize) -> Self {
        // Fill the entire grid with the base texture (e.g. grass).
        let tiles = vec![default_tile_id; width * height];
        
        Self {
            width,
            height,
            tile_size: 64.0,
            atlas_cols: 4, 
            atlas_rows: 37,
            tiles,
        }
    }

    /// Converts 2D (x,y) coordinates to a 1D index for the tiles array.
    pub fn get_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    /// Generates a single 3D mesh based on the current data in the `tiles` array.
pub fn generate_mesh(&self) -> Mesh {
        let num_tiles = self.width * self.height;
        let mut positions = Vec::with_capacity(num_tiles * 4);
        let mut normals = Vec::with_capacity(num_tiles * 4);
        let mut uvs = Vec::with_capacity(num_tiles * 4);
        
        // НОВІ МАСИВИ ДЛЯ ШЕЙДЕРА
        let mut blend_indices = Vec::with_capacity(num_tiles * 4);
        let mut blend_weights = Vec::with_capacity(num_tiles * 4);
        
        let mut indices = Vec::with_capacity(num_tiles * 6);
        let mut idx_offset = 0;

        for y in 0..self.height {
            for x in 0..self.width {
                let px = x as f32 * self.tile_size;
                let pz = y as f32 * self.tile_size;

                positions.push([px, 0.0, pz]);
                positions.push([px + self.tile_size, 0.0, pz]);
                positions.push([px + self.tile_size, 0.0, pz + self.tile_size]);
                positions.push([px, 0.0, pz + self.tile_size]);

                normals.extend_from_slice(&[[0.0, 1.0, 0.0]; 4]);

                let cell_index = self.get_index(x, y);
                let tile_id = self.tiles[cell_index] as u32;

                uvs.push([0.0, 0.0]);
                uvs.push([1.0, 0.0]);
                uvs.push([1.0, 1.0]);
                uvs.push([0.0, 1.0]);

                for _ in 0..4 {
                    blend_indices.push([tile_id, 0, 0, 0]);
                    blend_weights.push([1.0, 0.0, 0.0, 0.0]);
                }

                indices.extend_from_slice(&[
                    idx_offset, idx_offset + 2, idx_offset + 1,
                    idx_offset, idx_offset + 3, idx_offset + 2
                ]);
                idx_offset += 4;
            }
        }

        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        );
        
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        
        // ВСТАВЛЯЄМО НОВІ АТРИБУТИ В МЕШ
        mesh.insert_attribute(ATTRIBUTE_BLEND_INDICES, blend_indices);
        mesh.insert_attribute(ATTRIBUTE_BLEND_WEIGHTS, blend_weights);
        
        mesh.insert_indices(Indices::U32(indices));

        mesh
    }
}