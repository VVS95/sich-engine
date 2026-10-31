use bevy::color::Color;
use bevy::ecs::message::{MessageReader as EventReader, MessageWriter as EventWriter};
use bevy::math::primitives::Cuboid;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::events::CommandMoveUnit;
use crate::terrain::grid::{HEIGHT_SCALE, TerrainGrid};

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<CommandMoveUnit>()
            .add_systems(Startup, spawn_test_unit)
            .add_systems(
                Update,
                (
                    handle_mouse_input_system,
                    visual_sync_system,
                    unit_selection_system,
                    draw_selection_rings_system,
                ),
            )
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

#[derive(Component)]
pub struct Selected;

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
// Unit spawn
// ==========================================
fn spawn_test_unit(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Unit,
        LogicalPosition { x: 20000, z: 20000 },
        UnitStats { speed: 250 }, // Зробив його трохи швидшим
        Mesh3d(meshes.add(Cuboid::new(6.0, 8.0, 6.0))),
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
    selected_units: Query<Entity, With<Selected>>,
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
    for entity in &selected_units {
        ev_move.write(CommandMoveUnit {
            entity,
            target_x: (intersection.x * HEIGHT_SCALE).round() as i32,
            target_z: (intersection.z * HEIGHT_SCALE).round() as i32,
        });
    }
}

pub fn unit_selection_system(
    buttons: Res<ButtonInput<MouseButton>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    grid_query: Query<&TerrainGrid>,
    brush: Res<crate::interaction::TerrainBrush>,
    mut commands: Commands,
    units: Query<(Entity, &Transform, Has<Selected>), With<Unit>>,
    terrain_transform_query: Query<&GlobalTransform, With<TerrainGrid>>,
    mut drag_start: Local<Option<Vec2>>,
    mut contexts: EguiContexts,
) {
    if brush.mode != crate::interaction::BrushMode::None {
        *drag_start = None;
        return;
    }

    let Some(cursor_position) = windows.iter().find_map(Window::cursor_position) else {
        return;
    };

    if buttons.just_pressed(MouseButton::Left) {
        *drag_start = Some(cursor_position);
    }

    if buttons.pressed(MouseButton::Left) && let Some(start) = *drag_start {
        let Ok(context) = contexts.ctx_mut() else {
            return;
        };
        let painter = context.layer_painter(egui::LayerId::new(
            egui::Order::Tooltip,
            egui::Id::new("selection_box"),
        ));
        let rect = egui::Rect::from_two_pos(
            egui::pos2(start.x, start.y),
            egui::pos2(cursor_position.x, cursor_position.y),
        );
        painter.rect(
            rect,
            0.0,
            egui::Color32::from_rgba_unmultiplied(0, 255, 0, 30),
            egui::Stroke::new(1.0, egui::Color32::GREEN),
            egui::StrokeKind::Outside,
        );
    }

    if buttons.just_released(MouseButton::Left) && let Some(start) = *drag_start {
        let is_drag = start.distance(cursor_position) > 5.0;

        if is_drag {
            let min_x = start.x.min(cursor_position.x);
            let max_x = start.x.max(cursor_position.x);
            let min_y = start.y.min(cursor_position.y);
            let max_y = start.y.max(cursor_position.y);

            let Some((camera, camera_transform)) = cameras.iter().next() else {
                *drag_start = None;
                return;
            };

            for (entity, transform, is_selected) in &units {
                if let Ok(screen_pos) =
                    camera.world_to_viewport(camera_transform, transform.translation)
                {
                    if screen_pos.x >= min_x
                        && screen_pos.x <= max_x
                        && screen_pos.y >= min_y
                        && screen_pos.y <= max_y
                    {
                        if is_selected {
                            commands.entity(entity).remove::<Selected>();
                        } else {
                            commands.entity(entity).insert(Selected);
                        }
                    } else {
                        commands.entity(entity).remove::<Selected>();
                    }
                }
            }
        } else {
            let Some((camera, camera_transform)) = cameras.iter().next() else {
                *drag_start = None;
                return;
            };
            let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
                *drag_start = None;
                return;
            };
            let Some(grid) = grid_query.iter().next() else {
                *drag_start = None;
                return;
            };
            let Some(terrain_transform) = terrain_transform_query.iter().next() else {
                *drag_start = None;
                return;
            };
            let Some(intersection) =
                raycast_terrain(ray.origin, ray.direction.as_vec3(), terrain_transform, grid)
            else {
                *drag_start = None;
                return;
            };

            for (entity, transform, is_selected) in &units {
                if transform.translation.distance(intersection) < 8.0 {
                    if is_selected {
                        commands.entity(entity).remove::<Selected>();
                    } else {
                        commands.entity(entity).insert(Selected);
                    }
                } else {
                    commands.entity(entity).remove::<Selected>();
                }
            }
        }

        *drag_start = None;
    }
}

pub fn draw_selection_rings_system(mut gizmos: Gizmos, query: Query<&Transform, With<Selected>>) {
    for transform in &query {
        gizmos.circle(
            Isometry3d::new(
                transform.translation + Vec3::Y * 0.1,
                Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
            ),
            6.0,
            bevy::color::palettes::css::LIMEGREEN,
        );
    }
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
