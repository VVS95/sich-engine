mod world;
mod render;

use macroquad::prelude::*;
use macroquad::ui::{root_ui, widgets, hash};
use std::collections::HashMap;
use std::path::PathBuf;
use render::terrain_renderer::generate_terrain_meshes;
use world::camera::{map_to_screen, screen_to_map, zoom_for_preset, Camera, MAX_ZOOM_PRESET, MIN_ZOOM_PRESET, ZOOM_SMOOTHING};
use world::grid::TerrainGrid;

const TILE_WIDTH: f32 = 64.0;
const TILE_HEIGHT: f32 = 32.0;

fn point_in_triangle(px: f32, py: f32, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) -> bool {
    let denominator = (y2 - y3) * (x1 - x3) + (x3 - x2) * (y1 - y3);
    if denominator == 0.0 { return false; }
    let a = ((y2 - y3) * (px - x3) + (x3 - x2) * (py - y3)) / denominator;
    let b = ((y3 - y1) * (px - x3) + (x1 - x3) * (py - y3)) / denominator;
    let c = 1.0 - a - b;
    a >= 0.0 && a <= 1.0 && b >= 0.0 && b <= 1.0 && c >= 0.0 && c <= 1.0
}

fn apply_brush(grid: &mut TerrainGrid, center_x: usize, center_y: usize, radius: isize, strength: f32, raise: bool) {
    if radius <= 0 {
        return;
    }

    let radius_f = radius as f32;
    let center_x = center_x as isize;
    let center_y = center_y as isize;

    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let nx = center_x + dx;
            let ny = center_y + dy;

            if nx < 0 || ny < 0 || nx >= grid.width as isize || ny >= grid.height as isize {
                continue;
            }

            let distance = ((dx * dx + dy * dy) as f32).sqrt();
            if distance > radius_f {
                continue;
            }

            let effect = 1.0 - (distance / radius_f);
            let change = (effect * strength).round() as i16;
            if change == 0 {
                continue;
            }

            let current_height = grid.get_height(nx as usize, ny as usize).unwrap_or(0) as i16;
            let next_height = if raise {
                current_height.saturating_add(change)
            } else {
                current_height.saturating_sub(change)
            }
            .clamp(0, 255) as u8;

            grid.set_height(nx as usize, ny as usize, next_height);
        }
    }
}

struct AssetManager {
    base_path: PathBuf,
    textures_cache: HashMap<String, Texture2D>,
}

impl AssetManager {
    fn new(base_path: &str) -> Self {
        Self {
            base_path: PathBuf::from(base_path),
            textures_cache: HashMap::new(),
        }
    }
    pub fn load_palette(&self, pal_path: &str) -> Vec<[u8; 3]> {
        let full_path = self.base_path.join(pal_path);
        match std::fs::read(&full_path) {
            Ok(bytes) => {
                if bytes.len() < 768 {
                    eprintln!("Palette file '{}' too small ({} bytes)", full_path.display(), bytes.len());
                    return (0..256).map(|i| [i as u8, i as u8, i as u8]).collect();
                }

                let mut palette = Vec::with_capacity(256);
                for i in 0..256 {
                    let base = i * 3;
                    palette.push([bytes[base], bytes[base + 1], bytes[base + 2]]);
                }
                palette
            }
            Err(e) => {
                eprintln!("Failed to read palette '{}': {}", full_path.display(), e);
                (0..256).map(|i| [i as u8, i as u8, i as u8]).collect()
            }
        }
    }

    pub async fn load_texture(&mut self, virtual_path: &str, pal_path: Option<&str>) {
        let cache_key = if let Some(p) = pal_path { format!("{}?pal={}", virtual_path, p) } else { virtual_path.to_string() };
        if self.textures_cache.contains_key(&cache_key) {
            return;
        }

        let full_path = self.base_path.join(virtual_path);
        let bytes = match std::fs::read(&full_path) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Failed to read file from path '{}': {}", full_path.display(), e);
                return;
            }
        };

        if let Some(pal) = pal_path {
            let palette = self.load_palette(pal);

            // 1. Read basic BMP headers safely
            if bytes.len() < 54 {
                eprintln!("BMP file too small: {}", full_path.display());
                return;
            }

            // Pixel data offset is at byte 10
            let data_offset = u32::from_le_bytes([bytes[10], bytes[11], bytes[12], bytes[13]]) as usize;
            // Width is at byte 18, Height is at byte 22
            let width = i32::from_le_bytes([bytes[18], bytes[19], bytes[20], bytes[21]]) as usize;
            let height = i32::from_le_bytes([bytes[22], bytes[23], bytes[24], bytes[25]]).abs() as usize; // Height can be negative in BMP

            let mut rgba_buf = vec![0u8; width * height * 4];

            // BMP rows are padded to multiples of 4 bytes
            let row_stride = (width + 3) & !3;

            for y in 0..height {
                // Standard BMPs are stored bottom-up
                let src_row = height - 1 - y;
                let row_start = data_offset + (src_row * row_stride);

                for x in 0..width {
                    if row_start + x < bytes.len() {
                        let idx = bytes[row_start + x]; // The pure, unaltered GSC index!
                        let p = &palette[idx as usize];

                        let dst_idx = (y * width + x) * 4;
                        rgba_buf[dst_idx] = p[0];     // R
                        rgba_buf[dst_idx + 1] = p[1]; // G
                        rgba_buf[dst_idx + 2] = p[2]; // B
                        rgba_buf[dst_idx + 3] = 255;  // Alpha
                    }
                }
            }

            let texture = Texture2D::from_rgba8(width as u16, height as u16, &rgba_buf);
            texture.set_filter(FilterMode::Nearest);
            self.textures_cache.insert(cache_key, texture);
        } else {
            // Fallback: load as normal RGBA image
            let img = match image::load_from_memory(&bytes) {
                Ok(i) => i,
                Err(e) => {
                    eprintln!("Failed to decode image '{}': {}", full_path.display(), e);
                    return;
                }
            };
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();
            let texture = Texture2D::from_rgba8(width as u16, height as u16, &rgba.into_raw());
            texture.set_filter(FilterMode::Nearest);
            self.textures_cache.insert(cache_key, texture);
        }
    }

    fn get_texture(&self, virtual_path: &str) -> &Texture2D {
        self.textures_cache
            .get(virtual_path)
            .unwrap_or_else(|| panic!("Texture is not loaded: {}", virtual_path))
    }
}

#[macroquad::main("Cossacks Reborn")]
async fn main() {
    let mut game_map = TerrainGrid::new(256, 256);
    let mut asset_manager = AssetManager::new("C:/GSCExtractor/GSC File Utility/extracted");
    asset_manager.load_texture("TILES3.BMP", Some("AGEW_1.PAL")).await;
    let mut texture_mode = false;

    let mut brush_tex_id: f32 = 40.0;
    let mut base_tex_id: f32 = 12.0;
        
    let mut camera = Camera {
        offset_x: 400.0,
        offset_y: 100.0,
        speed: 800.0,
        zoom_level: zoom_for_preset(1),
        target_zoom_level: zoom_for_preset(1),
        zoom_preset: 1,
    };
    
    loop {
        clear_background(BLACK);
        let dt = get_frame_time(); 
        let (mouse_x, mouse_y) = mouse_position();
        let anchor_map = screen_to_map(mouse_x, mouse_y, &camera);

        if is_key_pressed(KeyCode::T) {
            texture_mode = !texture_mode;
        }

        let (_, wheel_y) = mouse_wheel();
        if wheel_y > 0.0 {
            camera.zoom_preset = (camera.zoom_preset - 1).max(MIN_ZOOM_PRESET);
            camera.target_zoom_level = zoom_for_preset(camera.zoom_preset);
        } else if wheel_y < 0.0 {
            camera.zoom_preset = (camera.zoom_preset + 1).min(MAX_ZOOM_PRESET);
            camera.target_zoom_level = zoom_for_preset(camera.zoom_preset);
        }

        let zoom_t = 1.0 - (-ZOOM_SMOOTHING * dt).exp();
        let previous_zoom = camera.zoom_level;
        camera.zoom_level += (camera.target_zoom_level - camera.zoom_level) * zoom_t;
        if (camera.zoom_level - previous_zoom).abs() > f32::EPSILON {
            let half_w = TILE_WIDTH / 2.0;
            let half_h = TILE_HEIGHT / 2.0;
            camera.offset_x = mouse_x - (anchor_map.0 - anchor_map.1) * half_w * camera.zoom_level;
            camera.offset_y = mouse_y - (anchor_map.0 + anchor_map.1) * half_h * camera.zoom_level;
        }

        let pan_speed = camera.speed * dt * camera.zoom_level;
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) { camera.offset_x -= pan_speed; }
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) { camera.offset_x += pan_speed; }
        if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) { camera.offset_y += pan_speed; }
        if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) { camera.offset_y -= pan_speed; }

        let sw = screen_width();
        let sh = screen_height();

        let corners = [
            screen_to_map(0.0, 0.0, &camera), 
            screen_to_map(sw, 0.0, &camera),  
            screen_to_map(sw, sh, &camera),   
            screen_to_map(0.0, sh, &camera),  
        ];

        let mut min_x = corners[0].0;
        let mut max_x = corners[0].0;
        let mut min_y = corners[0].1;
        let mut max_y = corners[0].1;

        for &(cx, cy) in &corners {
            if cx < min_x { min_x = cx; }
            if cx > max_x { max_x = cx; }
            if cy < min_y { min_y = cy; }
            if cy > max_y { max_y = cy; }
        }

        let padding = 15.0; 
        let start_x = (min_x - padding).max(0.0) as usize;
        let end_x = ((max_x + padding) as usize).min(game_map.width - 1);
        let start_y = (min_y - padding).max(0.0) as usize;
        let end_y = ((max_y + padding) as usize).min(game_map.height - 1);
        let terrain_tex = asset_manager.get_texture("TILES3.BMP?pal=AGEW_1.PAL");

        // UI: terrain editor
        root_ui().window(hash!(), vec2(10.0, 130.0), vec2(250.0, 150.0), |ui| {
            ui.label(None, "Terrain Editor");
            ui.separator();
            ui.label(None, &format!("Brush Texture: {}", brush_tex_id.round()));
            ui.slider(hash!(), "Brush Texture", 0.0..147.0, &mut brush_tex_id);
            // Force integer value immediately after slider interaction
            brush_tex_id = brush_tex_id.round();

            if ui.button(None, "Fill Map with Brush Texture") {
                base_tex_id = brush_tex_id.round();
                game_map.clear_sand_weights();
            }
        });

        // 1. TERRAIN GENERATION AND RENDERING
        let terrain_meshes = generate_terrain_meshes(
            &game_map,
            &camera,
            start_x,
            end_x,
            start_y,
            end_y,
            base_tex_id.round() as u8,
            brush_tex_id.round() as u8,
        );

        for mesh in terrain_meshes {
            let mut mesh = mesh.mesh;
            mesh.texture = Some(terrain_tex.weak_clone());
            draw_mesh(&mesh);
        }

        // 2. SMART 3D RAYCAST FOR THE MOUSE
        let base_x = anchor_map.0.floor() as isize;
        let base_y = anchor_map.1.floor() as isize;

        let mut hovered_tile: Option<(usize, usize)> = None;
        let search_radius = 6; 

        for dy in -search_radius..=search_radius {
            for dx in -search_radius..=search_radius {
                let check_x = base_x + dx;
                let check_y = base_y + dy;

                if check_x >= 0 && check_x < game_map.width as isize && check_y >= 0 && check_y < game_map.height as isize {
                    let cx = check_x as usize;
                    let cy = check_y as usize;

                    let h00 = game_map.get_height(cx, cy).unwrap_or(0) as f32;
                    let h10 = game_map.get_height(cx + 1, cy).unwrap_or(h00 as u8) as f32;
                    let h11 = game_map.get_height(cx + 1, cy + 1).unwrap_or(h00 as u8) as f32;
                    let h01 = game_map.get_height(cx, cy + 1).unwrap_or(h00 as u8) as f32;

                    let p00 = map_to_screen(cx as f32, cy as f32, h00, &camera);
                    let p10 = map_to_screen((cx + 1) as f32, cy as f32, h10, &camera);
                    let p11 = map_to_screen((cx + 1) as f32, (cy + 1) as f32, h11, &camera);
                    let p01 = map_to_screen(cx as f32, (cy + 1) as f32, h01, &camera);

                    if point_in_triangle(mouse_x, mouse_y, p00.0, p00.1, p10.0, p10.1, p11.0, p11.1) ||
                       point_in_triangle(mouse_x, mouse_y, p00.0, p00.1, p11.0, p11.1, p01.0, p01.1) {
                        hovered_tile = Some((cx, cy));
                    }
                }
            }
        }

        // 3. INTERACTION WITH THE HOVERED TILE
        if let Some((tx, ty)) = hovered_tile {
            let brush_radius: isize = 3;
            let num_dots = 36;
            let radius_f = brush_radius as f32;
            let center_x = tx as f32 + 0.5;
            let center_y = ty as f32 + 0.5;

            for i in 0..num_dots {
                let angle = i as f32 * (std::f32::consts::TAU / num_dots as f32);
                let px = center_x + radius_f * angle.cos();
                let py = center_y + radius_f * angle.sin();

                let sample_x = px.round().clamp(0.0, (game_map.width.saturating_sub(1)) as f32) as usize;
                let sample_y = py.round().clamp(0.0, (game_map.height.saturating_sub(1)) as f32) as usize;
                let sampled_height = game_map.get_height(sample_x, sample_y).unwrap_or(0) as f32;

                let (screen_x, screen_y) = map_to_screen(px, py, sampled_height, &camera);
                draw_circle(screen_x, screen_y, 1.5, YELLOW);
            }

            let h00 = game_map.get_height(tx, ty).unwrap_or(0) as f32;
            let p00 = map_to_screen(tx as f32, ty as f32, h00, &camera);

            draw_text(
                &format!("[{}, {}] H: {}", tx, ty, h00),
                p00.0 + 10.0,
                p00.1,
                20.0,
                YELLOW,
            );

            if !root_ui().is_mouse_captured() {
                if is_mouse_button_down(MouseButton::Left) {
                    if texture_mode {
                        game_map.add_sand_weight_in_radius(tx, ty, brush_radius, 0.1);
                    } else {
                        apply_brush(&mut game_map, tx, ty, brush_radius, 2.0, true);
                    }
                }
                if is_mouse_button_down(MouseButton::Right) {
                    apply_brush(&mut game_map, tx, ty, brush_radius, 2.0, false);
                }
            }
        }

        // UI
        draw_text(&format!("FPS: {}", get_fps()), 10.0, 30.0, 30.0, WHITE);
        draw_text(&format!("Visible Tiles: {}x{}", end_x - start_x, end_y - start_y), 10.0, 60.0, 30.0, WHITE);
        draw_text(
            if texture_mode { "Mode: Texture" } else { "Mode: Height" },
            10.0,
            90.0,
            30.0,
            WHITE,
        );
        
        next_frame().await
    }
}