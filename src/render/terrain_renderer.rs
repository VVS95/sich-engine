use macroquad::prelude::*;
use crate::world::camera;
use crate::world::grid::TerrainGrid;

pub struct TerrainRenderMesh {
    pub mesh: Mesh,
}

pub fn generate_terrain_meshes(
    grid: &TerrainGrid,
    camera: &crate::world::camera::Camera,
    start_x: usize,
    end_x: usize,
    start_y: usize,
    end_y: usize,
    base_tex_id: u8,
    brush_tex_id: u8,
) -> Vec<TerrainRenderMesh> {
    let mut meshes = Vec::new();

    if grid.width < 2 || grid.height < 2 || grid.heights.is_empty() {
        return meshes;
    }

    let start_x = start_x.min(grid.width.saturating_sub(1));
    let end_x = end_x.min(grid.width.saturating_sub(1));
    let start_y = start_y.min(grid.height.saturating_sub(1));
    let end_y = end_y.min(grid.height.saturating_sub(1));

    if start_x >= end_x || start_y >= end_y {
        return meshes;
    }

    let mut build_layer = |texture_id: usize, use_sand_alpha: bool| {
        let mut current_mesh = Mesh {
            vertices: Vec::new(),
            indices: Vec::new(),
            texture: None,
        };

        for ny in start_y..end_y {
            for nx in start_x..end_x {
                if current_mesh.indices.len() >= 3000 {
                    meshes.push(TerrainRenderMesh { mesh: current_mesh });
                    current_mesh = Mesh {
                        vertices: Vec::new(),
                        indices: Vec::new(),
                        texture: None,
                    };
                }

                let h_top = grid.get_height(nx, ny).unwrap_or(0);
                let h_right = grid.get_height(nx + 1, ny).unwrap_or(h_top);
                let h_bottom = grid.get_height(nx + 1, ny + 1).unwrap_or(h_top);
                let h_left = grid.get_height(nx, ny + 1).unwrap_or(h_top);

                let top_ws = (nx as f32, ny as f32, h_top as f32);
                let right_ws = ((nx + 1) as f32, ny as f32, h_right as f32);
                let bottom_ws = ((nx + 1) as f32, (ny + 1) as f32, h_bottom as f32);
                let left_ws = (nx as f32, (ny + 1) as f32, h_left as f32);

                let top_ss = camera::map_to_screen(top_ws.0, top_ws.1, top_ws.2, camera);
                let right_ss = camera::map_to_screen(right_ws.0, right_ws.1, right_ws.2, camera);
                let bottom_ss = camera::map_to_screen(bottom_ws.0, bottom_ws.1, bottom_ws.2, camera);
                let left_ss = camera::map_to_screen(left_ws.0, left_ws.1, left_ws.2, camera);

                // Higher-quality deterministic hash to avoid diagonal artifacts
                let n = (nx as u32).wrapping_mul(3266489917).wrapping_add((ny as u32).wrapping_mul(2654435761));
                let hash = (n ^ (n >> 15)) % 100;

                // Determine actual texture id for this tile depending on layer
                let (col, row) = if !use_sand_alpha {
                    // Base layer (grass/dirt) - derive actual_base_id from texture_id
                    let mut actual_base_id = texture_id;
                    if actual_base_id == 110 {
                        // #MULTI 110 109 110 108 (50% 110, 25% 109, 25% 108)
                        actual_base_id = if hash < 50 { 110 } else if hash < 75 { 109 } else { 108 };
                    } else if actual_base_id == 143 {
                        // #MULTI 143 139 143 139 (50% 143, 50% 139)
                        actual_base_id = if hash < 50 { 143 } else { 139 };
                    } else if actual_base_id == 142 {
                        // #MULTI 142 139 142 139 (50% 142, 50% 139)
                        actual_base_id = if hash < 50 { 142 } else { 139 };
                    } else if actual_base_id == 81 {
                        // #MULTI 81 82 81 80 (50% 81, 25% 82, 25% 80)
                        actual_base_id = if hash < 50 { 81 } else if hash < 75 { 82 } else { 80 };
                    }
                    (actual_base_id % 4, actual_base_id / 4)
                } else {
                    // Splat/brush layer - derive actual_brush_id from texture_id
                    let mut actual_brush_id = texture_id;
                    if actual_brush_id == 110 {
                        actual_brush_id = if hash < 50 { 110 } else if hash < 75 { 109 } else { 108 };
                    } else if actual_brush_id == 143 {
                        actual_brush_id = if hash < 50 { 143 } else { 139 };
                    } else if actual_brush_id == 142 {
                        actual_brush_id = if hash < 50 { 142 } else { 139 };
                    } else if actual_brush_id == 81 {
                        actual_brush_id = if hash < 50 { 81 } else if hash < 75 { 82 } else { 80 };
                    }
                    (actual_brush_id % 4, actual_brush_id / 4)
                };
                let u_step = 64.0 / 256.0;
                let v_step = 64.0 / 2368.0;
                let u0 = col as f32 * u_step;
                let v0 = row as f32 * v_step;
                let u1 = (col as f32 + 1.0) * u_step;
                let v1 = (row as f32 + 1.0) * v_step;

                let sand_alpha_top = if use_sand_alpha { grid.get_sand_weight(nx, ny) } else { 1.0 };
                let sand_alpha_right = if use_sand_alpha { grid.get_sand_weight(nx + 1, ny) } else { 1.0 };
                let sand_alpha_bottom = if use_sand_alpha { grid.get_sand_weight(nx + 1, ny + 1) } else { 1.0 };
                let sand_alpha_left = if use_sand_alpha { grid.get_sand_weight(nx, ny + 1) } else { 1.0 };

                if use_sand_alpha && sand_alpha_top <= 0.0 && sand_alpha_right <= 0.0 && sand_alpha_bottom <= 0.0 && sand_alpha_left <= 0.0 {
                    continue;
                }

                let i = current_mesh.vertices.len() as u16;
                let color_top = if use_sand_alpha { Color::new(1.0, 1.0, 1.0, sand_alpha_top) } else { WHITE };
                let color_right = if use_sand_alpha { Color::new(1.0, 1.0, 1.0, sand_alpha_right) } else { WHITE };
                let color_bottom = if use_sand_alpha { Color::new(1.0, 1.0, 1.0, sand_alpha_bottom) } else { WHITE };
                let color_left = if use_sand_alpha { Color::new(1.0, 1.0, 1.0, sand_alpha_left) } else { WHITE };

                current_mesh.vertices.push(Vertex::new(top_ss.0, top_ss.1, 0.0, u0, v0, color_top));
                current_mesh.vertices.push(Vertex::new(right_ss.0, right_ss.1, 0.0, u1, v0, color_right));
                current_mesh.vertices.push(Vertex::new(bottom_ss.0, bottom_ss.1, 0.0, u1, v1, color_bottom));
                current_mesh.vertices.push(Vertex::new(left_ss.0, left_ss.1, 0.0, u0, v1, color_left));

                current_mesh.indices.extend_from_slice(&[i, i + 1, i + 2, i, i + 2, i + 3]);
            }
        }

        if !current_mesh.vertices.is_empty() {
            meshes.push(TerrainRenderMesh { mesh: current_mesh });
        }
    };

    build_layer(base_tex_id as usize, false);
    build_layer(brush_tex_id as usize, true);

    meshes
}