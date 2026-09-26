use crate::terrain::grid::TerrainGrid;
use bevy::prelude::*;

// --- COMPONENTS & RESOURCES ---

/// Resource holding the current 3D world position of the mouse cursor on the terrain plane.
#[derive(Resource, Default)]
pub struct CursorWorldPosition(pub Option<Vec3>);

/// Configuration for the terrain painting tool.
#[derive(Resource)]
pub struct TerrainBrush {
    pub radius: f32,
    pub tile_id: usize,
    pub is_active: bool,
}

impl Default for TerrainBrush {
    fn default() -> Self {
        Self {
            radius: 96.0,
            tile_id: 20,
            is_active: true,
        }
    }
}

// --- PLUGIN ---

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorWorldPosition>()
            .init_resource::<TerrainBrush>()
            .add_systems(
                Update,
                (
                    update_cursor_world_position,
                    apply_brush_directly,
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

/// Пряма система малювання: читає клік миші, змінює сітку та одразу оновлює меш.
pub fn apply_brush_directly(
    mouse_input: Res<ButtonInput<MouseButton>>,
    cursor_pos: Res<CursorWorldPosition>,
    brush: Res<TerrainBrush>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut terrain_query: Query<(&GlobalTransform, &mut TerrainGrid, &Mesh3d)>,
) {
    if !brush.is_active || !mouse_input.pressed(MouseButton::Left) {
        return;
    }

    let Some(world_pos) = cursor_pos.0 else {
        return;
    };

    let radius_squared = brush.radius * brush.radius;
    let mut grid_modified = false;

    for (terrain_transform, mut grid, _) in &mut terrain_query {
        let local_pos = terrain_transform
            .affine()
            .inverse()
            .transform_point3(world_pos);

        for y in 0..grid.height {
            for x in 0..grid.width {
                let center = Vec2::new(
                    (x as f32 + 0.5) * grid.tile_size,
                    (y as f32 + 0.5) * grid.tile_size,
                );
                let distance = center - Vec2::new(local_pos.x, local_pos.z);

                if distance.length_squared() <= radius_squared {
                    let index = grid.get_index(x, y);
                    if grid.tiles[index] != brush.tile_id {
                        grid.tiles[index] = brush.tile_id;
                        grid_modified = true;
                    }
                }
            }
        }
    }

    if grid_modified {
        info!("Grid modified by brush! Rebuilding mesh...");
        for (_, grid, mesh3d) in &terrain_query {
            let new_mesh = grid.generate_mesh();
            if let Err(err) = meshes.insert(&mesh3d.0, new_mesh) {
                error!("Failed to update terrain mesh: {err:?}");
            }
        }
    }
}