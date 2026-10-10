use crate::terrain::{grid::TerrainGrid, splatmap::TerrainSplatmap};
use bevy::prelude::*;
use std::f32::consts::FRAC_PI_2;

// --- COMPONENTS & RESOURCES ---

/// Resource holding the current 3D world position of the mouse cursor on the terrain plane.
#[derive(Resource, Default)]
pub struct CursorWorldPosition(pub Option<Vec3>);

/// World position from the previous painted frame, used to fill fast mouse paths.
#[derive(Resource, Default)]
pub struct PreviousBrushPosition(pub Option<Vec3>);

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum BrushMode {
    Paint,
    Raise,
    Lower,
}

/// Configuration for the terrain painting tool.
#[derive(Resource)]
pub struct TerrainBrush {
    pub radius: f32,
    pub tile_id: usize,
    pub is_active: bool,
    pub mode: BrushMode,
    pub over_ui: bool,
}

impl Default for TerrainBrush {
    fn default() -> Self {
        Self {
            radius: 96.0,
            tile_id: 1,
            is_active: true,
            mode: BrushMode::Paint,
            over_ui: false,
        }
    }
}

// --- PLUGIN ---

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorWorldPosition>()
            .init_resource::<PreviousBrushPosition>()
            .init_resource::<TerrainBrush>()
            .add_systems(
                Update,
                (
                    update_cursor_world_position,
                    apply_brush_directly,
                    draw_brush_cursor,
                )
                    .chain(), // НАЙГОЛОВНІШЕ: Гарантуємо порядок виконання кадрів!
            );
    }
}

// --- SYSTEMS ---

pub fn update_cursor_world_position(
    window_query: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut cursor_pos: ResMut<CursorWorldPosition>,
) {
    let Ok(window) = window_query.single() else {
        cursor_pos.0 = None;
        return;
    };
    let Some(screen_pos) = window.cursor_position() else {
        cursor_pos.0 = None;
        return;
    };
    let Ok((camera, camera_transform)) = camera_query.single() else {
        cursor_pos.0 = None;
        return;
    };
    let Ok(ray) = camera.viewport_to_world(camera_transform, screen_pos) else {
        cursor_pos.0 = None;
        return;
    };

    let direction = ray.direction.as_vec3();
    if direction.y.abs() < f32::EPSILON {
        cursor_pos.0 = None;
        return;
    }

    let distance = -ray.origin.y / direction.y;
    cursor_pos.0 = (distance >= 0.0).then_some(ray.origin + direction * distance);
}

/// Draws the active brush radius on the terrain plane.
pub fn draw_brush_cursor(
    mut gizmos: Gizmos,
    cursor_pos: Res<CursorWorldPosition>,
    brush: Res<TerrainBrush>,
) {
    if !brush.is_active {
        return;
    }

    let Some(pos) = cursor_pos.0 else {
        return;
    };

    let isometry = Isometry3d::new(
        Vec3::new(pos.x, pos.y + 0.1, pos.z),
        Quat::from_rotation_x(-FRAC_PI_2),
    );
    gizmos.circle(
        isometry,
        brush.radius,
        Color::srgba(0.0, 1.0, 1.0, 0.8),
    );
}

/// Paints the four-slot index and weight splatmaps directly in the main world.
pub fn apply_brush_directly(
    mouse_input: Res<ButtonInput<MouseButton>>,
    cursor_pos: Res<CursorWorldPosition>,
    brush: Res<TerrainBrush>,
    mut previous_pos: ResMut<PreviousBrushPosition>,
    splatmap: Res<TerrainSplatmap>,
    mut images: ResMut<Assets<Image>>,
    terrain_query: Query<(&GlobalTransform, &TerrainGrid)>,
) {
    if brush.over_ui {
        return;
    }

    if !brush.is_active || !mouse_input.pressed(MouseButton::Left) {
        previous_pos.0 = None;
        return;
    }

    let Some(world_pos) = cursor_pos.0 else {
        previous_pos.0 = None;
        return;
    };

    let start = previous_pos.0.unwrap_or(world_pos);
    let spacing = (brush.radius * 0.25).max(1.0);
    let steps = (start.distance(world_pos) / spacing).ceil() as usize;
    let Some(mut indices_data) = images
        .get(splatmap.indices.id())
        .and_then(|image| image.data.as_ref())
        .cloned()
    else {
        return;
    };
    let Some(mut weights_data) = images
        .get(splatmap.weights.id())
        .and_then(|image| image.data.as_ref())
        .cloned()
    else {
        return;
    };
    let resolution = splatmap.resolution;
    let Some(tile_id) = u8::try_from(brush.tile_id).ok() else {
        return;
    };

    for step in 0..=steps {
        let t = if steps == 0 {
            1.0
        } else {
            step as f32 / steps as f32
        };
        let sample_pos = start.lerp(world_pos, t);

        for (terrain_transform, grid) in &terrain_query {
            let local_pos = terrain_transform
                .affine()
                .inverse()
                .transform_point3(sample_pos);
            let terrain_size = Vec2::new(
                grid.width as f32 * grid.tile_size,
                grid.height as f32 * grid.tile_size,
            );
            let center_uv = Vec2::new(local_pos.x / terrain_size.x, local_pos.z / terrain_size.y);
            let pixels_per_world = Vec2::new(
                resolution.x as f32 / terrain_size.x,
                resolution.y as f32 / terrain_size.y,
            );
            let pixel_radius = brush.radius * pixels_per_world.x.max(pixels_per_world.y);
            let min_x = ((center_uv.x * resolution.x as f32 - pixel_radius).floor() as i32)
                .clamp(0, resolution.x as i32 - 1);
            let max_x = ((center_uv.x * resolution.x as f32 + pixel_radius).ceil() as i32)
                .clamp(0, resolution.x as i32 - 1);
            let min_y = ((center_uv.y * resolution.y as f32 - pixel_radius).floor() as i32)
                .clamp(0, resolution.y as i32 - 1);
            let max_y = ((center_uv.y * resolution.y as f32 + pixel_radius).ceil() as i32)
                .clamp(0, resolution.y as i32 - 1);

            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    let pixel_uv = Vec2::new(
                        (x as f32 + 0.5) / resolution.x as f32,
                        (y as f32 + 0.5) / resolution.y as f32,
                    );
                    let distance = (pixel_uv - center_uv) * terrain_size;
                    let distance_length = distance.length();
                    if distance_length > brush.radius {
                        continue;
                    }

                    let strength = 1.0 - (distance_length / brush.radius).clamp(0.0, 1.0);
                    let offset = ((y as u32 * resolution.x + x as u32) * 4) as usize;
                    let mut indices = [
                        indices_data[offset],
                        indices_data[offset + 1],
                        indices_data[offset + 2],
                        indices_data[offset + 3],
                    ];
                    let weights = [
                        weights_data[offset],
                        weights_data[offset + 1],
                        weights_data[offset + 2],
                        weights_data[offset + 3],
                    ];
                    let normalized = paint_splat_pixel(&mut indices, weights, tile_id, strength);

                    for channel in 0..4 {
                        indices_data[offset + channel] = indices[channel];
                        weights_data[offset + channel] = normalized[channel];
                    }
                }
            }
        }
    }

    if let Some(mut image) = images.get_mut(&splatmap.indices) {
        image.data = Some(indices_data);
    }
    if let Some(mut image) = images.get_mut(&splatmap.weights) {
        image.data = Some(weights_data);
    }

    previous_pos.0 = Some(world_pos);
}

fn paint_splat_pixel(
    indices: &mut [u8; 4],
    source_weights: [u8; 4],
    tile_id: u8,
    brush_strength: f32,
) -> [u8; 4] {
    let mut weights = source_weights.map(f32::from);
    let target_slot = if let Some(slot) = indices.iter().position(|&index| index == tile_id) {
        slot
    } else {
        let slot = weights
            .iter()
            .enumerate()
            .min_by_key(|(_, weight)| **weight as u8)
            .map(|(slot, _)| slot)
            .unwrap_or(0);
        indices[slot] = tile_id;
        weights[slot] = 0.0;
        slot
    };

    let requested_add = brush_strength.clamp(0.0, 1.0) * 255.0;
    let added = requested_add.min(255.0 - weights[target_slot]);
    let other_total: f32 = weights
        .iter()
        .enumerate()
        .filter(|(slot, _)| *slot != target_slot)
        .map(|(_, weight)| *weight)
        .sum();

    if other_total > 0.0 {
        for (slot, weight) in weights.iter_mut().enumerate() {
            if slot != target_slot {
                *weight -= added * (*weight / other_total);
            }
        }
    }
    weights[target_slot] += added;

    let weight_sum: f32 = weights.iter().sum();
    if weight_sum > 0.0 {
        for weight in &mut weights {
            *weight *= 255.0 / weight_sum;
        }
    } else {
        weights[target_slot] = 255.0;
    }

    let mut normalized = [0u8; 4];
    for (slot, weight) in weights.iter().enumerate() {
        normalized[slot] = weight.round().clamp(0.0, 255.0) as u8;
    }

    let sum: i32 = normalized.iter().map(|&weight| i32::from(weight)).sum();
    let difference = 255 - sum;
    if difference != 0 {
        let adjustment_slot = normalized
            .iter()
            .enumerate()
            .max_by_key(|(_, weight)| **weight)
            .map(|(slot, _)| slot)
            .unwrap_or(target_slot);
        normalized[adjustment_slot] =
            (i32::from(normalized[adjustment_slot]) + difference).clamp(0, 255) as u8;
    }

    normalized
}
