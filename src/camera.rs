use bevy::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
           .add_systems(Update, pan_camera);
    }
}

// Marker component for the main camera.
#[derive(Component)]
pub struct MainCamera;

// Create the camera entity.
fn spawn_camera(mut commands: Commands) {
    let position = Vec3::new(20.0, 25.0, 20.0);

    commands.spawn((
        Camera3dBundle {
            projection: Projection::Orthographic(OrthographicProjection {
                scale: 3.0, // Standard zoom.
                scaling_mode: bevy::render::camera::ScalingMode::FixedVertical(20.0),
                ..default()
            }),
            // Place the camera in the scene.
            transform: Transform::from_translation(position).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        MainCamera,
    ));
}

// Handle camera movement.
fn pan_camera(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<MainCamera>>, // Find the main camera transform.
) {
    // Get the camera transform.
    let mut transform = query.single_mut();

    // Movement speed.
    let speed = 25.0 * time.delta_seconds();
    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        direction.x -= 1.0;
        direction.z -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        direction.x += 1.0;
        direction.z += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
        direction.z += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
        direction.z -= 1.0;
    }

    if direction != Vec3::ZERO {
        transform.translation += direction.normalize() * speed;
    }
}