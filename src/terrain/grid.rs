use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

#[derive(Component)]
pub struct TerrainGrid {
    pub width: usize,
    pub height: usize,
    pub tile_size: f32,
    pub atlas_cols: usize,
    pub atlas_rows: usize,
}

impl TerrainGrid {
    pub fn new(width: usize, height: usize, _default_tile_id: usize) -> Self {
        Self {
            width,
            height,
            tile_size: 64.0,
            atlas_cols: 4,
            atlas_rows: 37,
        }
    }

    /// Converts 2D (x,y) coordinates to a 1D index for the tiles array.
    pub fn get_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    /// Generates terrain geometry with local atlas UVs and global splatmap UVs.
    pub fn generate_mesh(&self) -> Mesh {
        let num_tiles = self.width * self.height;
        let mut positions = Vec::with_capacity(num_tiles * 4);
        let mut normals = Vec::with_capacity(num_tiles * 4);
        let mut uvs = Vec::with_capacity(num_tiles * 4);

        // НОВІ МАСИВИ ДЛЯ ШЕЙДЕРА
        let mut splat_uvs = Vec::with_capacity(num_tiles * 4);

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

                uvs.push([0.0, 0.0]);
                uvs.push([1.0, 0.0]);
                uvs.push([1.0, 1.0]);
                uvs.push([0.0, 1.0]);

                let min_uv = [x as f32 / self.width as f32, y as f32 / self.height as f32];
                let max_uv = [
                    (x + 1) as f32 / self.width as f32,
                    (y + 1) as f32 / self.height as f32,
                ];
                splat_uvs.extend_from_slice(&[
                    [min_uv[0], min_uv[1]],
                    [max_uv[0], min_uv[1]],
                    [max_uv[0], max_uv[1]],
                    [min_uv[0], max_uv[1]],
                ]);

                indices.extend_from_slice(&[
                    idx_offset,
                    idx_offset + 2,
                    idx_offset + 1,
                    idx_offset,
                    idx_offset + 3,
                    idx_offset + 2,
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

        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_1, splat_uvs);

        mesh.insert_indices(Indices::U32(indices));

        mesh
    }
}
