use bevy::color::Color;
use bevy::ecs::message::{MessageReader as EventReader, MessageWriter as EventWriter};
use bevy::math::primitives::Cuboid;
use bevy::prelude::*;

use crate::events::CommandMoveUnit;
use crate::terrain::grid::{HEIGHT_SCALE, TerrainGrid};

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<CommandMoveUnit>()
            .add_systems(Startup, spawn_test_unit)
            .add_systems(Update, (handle_mouse_input_system, visual_sync_system))
            .add_systems(
                FixedUpdate,
                (execute_move_commands_system, logical_movement_system),
            );
    }
}

// ==========================================
// КОМПОНЕНТИ
// ==========================================
#[derive(Component, Debug, Clone, Copy)]
pub struct Unit;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerId(pub u8);

#[derive(Component, Debug, Clone, Copy)]
pub struct LogicalPosition {
    pub x: i32,
    pub z: i32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct MoveTarget {
    pub x: i32,
    pub z: i32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct UnitStats {
    pub speed: i32,
}

// ==========================================
// СПАВН ЮНІТА
// ==========================================
fn spawn_test_unit(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    info!(">>> СПАВН ЗЕЛЕНОГО ЮНІТА <<<");
    commands.spawn((
        Unit,
        LogicalPosition { x: 20000, z: 20000 },
        UnitStats { speed: 250 }, // Зробив його трохи швидшим
        Mesh3d(meshes.add(Cuboid::new(4.0, 8.0, 4.0))),
        // ЗЕЛЕНИЙ колір, щоб точно знати, що новий код працює!
        MeshMaterial3d(materials.add(Color::srgb(0.0, 1.0, 0.0))),
        Transform::from_translation(Vec3::new(20.0, 10.0, 20.0)),
    ));
}

// ==========================================
// СИСТЕМИ
// ==========================================
pub fn handle_mouse_input_system(
    buttons: Res<ButtonInput<MouseButton>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    mut ev_move: EventWriter<CommandMoveUnit>,
    units: Query<Entity, With<Unit>>,
    terrain_query: Query<&TerrainGrid>,
    terrain_transform_query: Query<&GlobalTransform, With<TerrainGrid>>,
) {
    if !buttons.just_pressed(MouseButton::Right) {
        return;
    }

    let Some(cursor_position) = windows.iter().find_map(Window::cursor_position) else {
        return;
    };
    let Some((camera, camera_transform)) = cameras.iter().next() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    let direction = ray.direction.as_vec3();
    let Some(grid) = terrain_query.iter().next() else {
        return;
    };
    let Some(terrain_transform) = terrain_transform_query.iter().next() else {
        return;
    };
    let Some(intersection) = raycast_terrain(ray.origin, direction, terrain_transform, grid) else {
        return;
    };
    let Some(entity) = units.iter().next() else {
        return;
    };

    ev_move.write(CommandMoveUnit {
        entity,
        target_x: (intersection.x * HEIGHT_SCALE).round() as i32,
        target_z: (intersection.z * HEIGHT_SCALE).round() as i32,
    });
}

fn raycast_terrain(
    origin: Vec3,
    direction: Vec3,
    terrain_transform: &GlobalTransform,
    grid: &TerrainGrid,
) -> Option<Vec3> {
    if direction.length_squared() <= f32::EPSILON || grid.width == 0 || grid.height == 0 {
        return None;
    }

    let direction = direction.normalize();
    let inverse = terrain_transform.affine().inverse();
    let local_origin = inverse.transform_point3(origin);
    let local_direction = inverse.transform_vector3(direction);
    if local_direction.length_squared() <= f32::EPSILON {
        return None;
    }

    let max_x = grid.width as f32 * grid.tile_size;
    let max_z = grid.height as f32 * grid.tile_size;
    let step = (grid.tile_size * 0.05).max(0.25);
    let max_distance = 20_000.0;
    let mut previous: Option<(f32, f32)> = None;
    let mut distance = 0.0;

    while distance <= max_distance {
        let point = local_origin + local_direction * distance;
        if point.x >= 0.0 && point.x <= max_x && point.z >= 0.0 && point.z <= max_z {
            let terrain_y = grid.get_height_at_world(point.x, point.z);
            let signed_height = point.y - terrain_y;

            if let Some((previous_distance, previous_signed_height)) = previous {
                if previous_signed_height >= 0.0 && signed_height <= 0.0 {
                    let mut low = previous_distance;
                    let mut high = distance;
                    for _ in 0..12 {
                        let middle = (low + high) * 0.5;
                        let middle_point = local_origin + local_direction * middle;
                        let middle_height =
                            grid.get_height_at_world(middle_point.x, middle_point.z);
                        if middle_point.y - middle_height > 0.0 {
                            low = middle;
                        } else {
                            high = middle;
                        }
                    }

                    let local_hit = local_origin + local_direction * high;
                    return Some(terrain_transform.transform_point(local_hit));
                }
            } else if signed_height <= 0.0 {
                return Some(terrain_transform.transform_point(point));
            }

            previous = Some((distance, signed_height));
        } else {
            previous = None;
        }

        distance += step;
    }

    None
}

pub fn execute_move_commands_system(
    mut commands: Commands,
    mut ev_move: EventReader<CommandMoveUnit>,
) {
    for ev in ev_move.read() {
        commands.entity(ev.entity).insert(MoveTarget {
            x: ev.target_x,
            z: ev.target_z,
        });
    }
}

pub fn logical_movement_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut LogicalPosition, &MoveTarget, &UnitStats)>,
) {
    for (ent, mut pos, target, stats) in &mut query {
        let dx = target.x as i64 - pos.x as i64;
        let dz = target.z as i64 - pos.z as i64;
        let dist = (dx * dx + dz * dz).isqrt();
        let speed = i64::from(stats.speed).max(0);

        if dist == 0 || dist <= speed {
            pos.x = target.x;
            pos.z = target.z;
            commands.entity(ent).remove::<MoveTarget>();
            continue;
        }

        if speed == 0 {
            continue;
        }

        let step_x = fixed_point_step(dx, speed, dist);
        let step_z = fixed_point_step(dz, speed, dist);
        pos.x = (pos.x as i64 + step_x).clamp(i32::MIN as i64, i32::MAX as i64) as i32;
        pos.z = (pos.z as i64 + step_z).clamp(i32::MIN as i64, i32::MAX as i64) as i32;
    }
}

fn fixed_point_step(component: i64, speed: i64, distance: i64) -> i64 {
    if component == 0 {
        0
    } else {
        let step = component * speed / distance;
        if step == 0 { component.signum() } else { step }
    }
}

pub fn visual_sync_system(
    time: Res<Time>,
    mut query: Query<(&LogicalPosition, &mut Transform), With<Unit>>,
    grid_query: Query<(&GlobalTransform, &TerrainGrid)>,
) {
    let Some((terrain_transform, grid)) = grid_query.iter().next() else {
        return;
    };

    for (pos, mut transform) in &mut query {
        let world_x = pos.x as f32 / HEIGHT_SCALE;
        let world_z = pos.z as f32 / HEIGHT_SCALE;
        let inverse = terrain_transform.affine().inverse();
        let local_position = inverse.transform_point3(Vec3::new(world_x, 0.0, world_z));
        let terrain_y = grid.get_height_at_world(local_position.x, local_position.z);
        if terrain_y.abs() > 0.01 {
            info!(
                "Unit Logic: ({}, {}), Terrain Y: {}",
                pos.x, pos.z, terrain_y
            );
        }

        let terrain_world_position = terrain_transform.transform_point(Vec3::new(
            local_position.x,
            terrain_y,
            local_position.z,
        ));
        let target_vec = Vec3::new(world_x, terrain_world_position.y + 4.0, world_z);

        transform.translation = transform
            .translation
            .lerp(target_vec, 1.0 - (-15.0 * time.delta_secs()).exp());
    }
}
