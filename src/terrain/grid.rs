use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};

#[derive(Component)]
pub struct TerrainGrid {
    pub width: usize,
    pub height: usize,
    pub tile_size: f32,
    pub atlas_cols: usize,
    pub atlas_rows: usize,
    pub heights: Vec<f32>,
}

impl TerrainGrid {
    pub fn new(width: usize, height: usize, _default_tile_id: usize) -> Self {
        Self {
            width,
            height,
            tile_size: 16.0,
            atlas_cols: 4,
            atlas_rows: 37,
            heights: vec![0.0; (width + 1) * (height + 1)],
        }
    }

    /// Converts 2D tile coordinates to a 1D index.
    pub fn get_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn calculate_normal(&self, x: usize, y: usize) -> [f32; 3] {
        let stride = self.width + 1;
        let x_left = x.saturating_sub(1);
        let x_right = (x + 1).min(self.width);
        let y_up = y.saturating_sub(1);
        let y_down = (y + 1).min(self.height);

        let h_left = self.heights[y * stride + x_left];
        let h_right = self.heights[y * stride + x_right];
        let h_up = self.heights[y_up * stride + x];
        let h_down = self.heights[y_down * stride + x];

        let normal =
            Vec3::new(h_left - h_right, 2.0 * self.tile_size, h_up - h_down).normalize_or_zero();

        normal.to_array()
    }

    /// Generates terrain geometry with one shared vertex for every height sample.
    pub fn generate_mesh(&self) -> Mesh {
        let vertex_count = (self.width + 1) * (self.height + 1);
        let mut positions = Vec::with_capacity(vertex_count);
        let mut normals = Vec::with_capacity(vertex_count);
        let mut uv_0 = Vec::with_capacity(vertex_count);
        let mut uv_1 = Vec::with_capacity(vertex_count);

        for y in 0..=self.height {
            for x in 0..=self.width {
                let index = y * (self.width + 1) + x;
                positions.push([
                    x as f32 * self.tile_size,
                    self.heights[index],
                    y as f32 * self.tile_size,
                ]);
                normals.push(self.calculate_normal(x, y));
                let uv = [x as f32 / self.width as f32, y as f32 / self.height as f32];
                uv_0.push(uv);
                uv_1.push(uv);
            }
        }

        let mut indices = Vec::with_capacity(self.width * self.height * 6);
        for y in 0..self.height {
            for x in 0..self.width {
                let top_left = y * (self.width + 1) + x;
                let top_right = top_left + 1;
                let bottom_left = (y + 1) * (self.width + 1) + x;
                let bottom_right = bottom_left + 1;

                indices.extend_from_slice(&[
                    top_left as u32,
                    bottom_right as u32,
                    top_right as u32,
                    top_left as u32,
                    bottom_left as u32,
                    bottom_right as u32,
                ]);
            }
        }

        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        );

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv_0);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_1, uv_1);
        mesh.insert_indices(Indices::U32(indices));

        mesh
    }
}

pub fn update_terrain_mesh(
    mut meshes: ResMut<Assets<Mesh>>,
    terrain_query: Query<(&TerrainGrid, &Mesh3d), Changed<TerrainGrid>>,
) {
    for (grid, mesh_handle) in &terrain_query {
        let Some(mut mesh) = meshes.get_mut(&mesh_handle.0) else {
            continue;
        };

        let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
        else {
            continue;
        };

        for (i, position) in positions.iter_mut().enumerate() {
            position[1] = grid.heights[i];
        }

        let Some(VertexAttributeValues::Float32x3(normals)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_NORMAL)
        else {
            continue;
        };

        for y in 0..=grid.height {
            for x in 0..=grid.width {
                let index = y * (grid.width + 1) + x;
                normals[index] = grid.calculate_normal(x, y);
            }
        }
    }
}
