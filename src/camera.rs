use bevy::{camera::ScalingMode, input::mouse::MouseWheel, prelude::*};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, update_camera);
    }
}

// Marker component for the main camera.
#[derive(Component)]
pub struct MainCamera;

// Desired camera state. The camera follows this state smoothly in update_camera.
#[derive(Component)]
pub struct CameraController {
    pub target_scale: f32,
    pub target_position: Vec3,
}

const MIN_ZOOM: f32 = 1.0;
const MAX_ZOOM: f32 = 10.0;
const ZOOM_STEP: f32 = 0.5;
const PAN_SPEED: f32 = 25.0;
const ZOOM_SMOOTHING: f32 = 12.0;
const POSITION_SMOOTHING: f32 = 12.0;

// Create the camera entity.
fn spawn_camera(mut commands: Commands) {
    let position = Vec3::new(50.0, 40.82, 50.0);

    commands.spawn((
        Camera3d::default(),
        Msaa::Off,
        Projection::Orthographic(OrthographicProjection {
            scale: 3.0,
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 20.0,
            },
            // ГОЛОВНЕ ВИПРАВЛЕННЯ: Розширюємо площини відсікання, 
            // щоб камера захоплювала ландшафт при будь-якому зумі
            near: -10000.0, 
            far: 10000.0,
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_translation(position).looking_at(Vec3::ZERO, Dir3::Y),
        CameraController {
            target_scale: 3.0,
            target_position: position,
        },
        MainCamera,
    ));
}

// Update desired camera state from input, then smoothly approach it.
fn update_camera(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    mut query: Query<(&mut Transform, &mut Projection, &mut CameraController), With<MainCamera>>,
) {
    // ВАЖЛИВО: Захист від крашу (panic). Замість single_mut() безпечніше використовувати get_single_mut()
    let Ok((mut transform, mut projection, mut controller)) = query.single_mut() else {
        return;
    };

    for event in mouse_wheel.read() {
        controller.target_scale =
            (controller.target_scale - event.y * ZOOM_STEP).clamp(MIN_ZOOM, MAX_ZOOM);
    }

    let speed = PAN_SPEED * controller.target_scale * time.delta_secs();
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
        controller.target_position += direction.normalize_or_zero() * speed;
    }

    let position_lerp = 1.0 - (-POSITION_SMOOTHING * time.delta_secs()).exp();
    transform.translation = transform
        .translation
        .lerp(controller.target_position, position_lerp);

    let Projection::Orthographic(orthographic) = projection.as_mut() else {
        return;
    };

    let zoom_lerp = 1.0 - (-ZOOM_SMOOTHING * time.delta_secs()).exp();
    orthographic.scale = orthographic.scale.lerp(controller.target_scale, zoom_lerp);
}
