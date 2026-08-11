const TILE_WIDTH: f32 = 64.0;
const TILE_HEIGHT: f32 = 32.0;
const Z_SCALE: f32 = 16.0;
pub const MIN_ZOOM_LEVEL: f32 = 0.2;
const MAX_ZOOM_LEVEL: f32 = 3.0;
pub const MIN_ZOOM_PRESET: i32 = 1;
pub const MAX_ZOOM_PRESET: i32 = 12;
const ZOOM_PRESET_STEP: f32 = 0.84;
pub const ZOOM_SMOOTHING: f32 = 12.0;

pub struct Camera {
    pub offset_x: f32,
    pub offset_y: f32,
    pub speed: f32,
    pub zoom_level: f32,
    pub target_zoom_level: f32,
    pub zoom_preset: i32,
}

pub fn zoom_for_preset(preset: i32) -> f32 {
    let steps_from_close = (preset - MIN_ZOOM_PRESET).max(0);
    (1.0 * ZOOM_PRESET_STEP.powi(steps_from_close)).clamp(MIN_ZOOM_LEVEL, MAX_ZOOM_LEVEL)
}

pub fn map_to_screen(map_x: f32, map_y: f32, map_z: f32, camera: &Camera) -> (f32, f32) {
    let zoom = camera.zoom_level;
    let screen_x = (map_x - map_y) * (TILE_WIDTH / 2.0) * zoom + camera.offset_x;
    let screen_y = ((map_x + map_y) * (TILE_HEIGHT / 2.0) - (map_z * Z_SCALE)) * zoom + camera.offset_y;
    (screen_x, screen_y)
}

pub fn screen_to_map(screen_x: f32, screen_y: f32, camera: &Camera) -> (f32, f32) {
    let zoom = camera.zoom_level.max(0.0001);
    let adj_x = (screen_x - camera.offset_x) / zoom;
    let adj_y = (screen_y - camera.offset_y) / zoom;
    let half_w = TILE_WIDTH / 2.0;
    let half_h = TILE_HEIGHT / 2.0;
    let map_x = (adj_x / half_w + adj_y / half_h) / 2.0;
    let map_y = (adj_y / half_h - adj_x / half_w) / 2.0;
    (map_x, map_y)
}