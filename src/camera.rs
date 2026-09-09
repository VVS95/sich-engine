use bevy::{camera::ScalingMode, prelude::*};

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
        // 1. У сучасних версіях (Required Components) достатньо просто передати компонент
        Camera3d::default(),
        
        // 2. Дефолтна 3D перспектива замінюється на ортогональну для ізометрії
        Projection::Orthographic(OrthographicProjection {
            scale: 3.0,
            scaling_mode: ScalingMode::FixedVertical { viewport_height: 20.0 },
            ..OrthographicProjection::default_3d() 
        }),
        
        // 3. Dir3::Y - правильний сучасний підхід для вектора "Вгору"
        Transform::from_translation(position).looking_at(Vec3::ZERO, Dir3::Y),
        
        MainCamera,
    ));
}

// Handle camera movement.
fn pan_camera(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<MainCamera>>,
) {
    // ВАЖЛИВО: Захист від крашу (panic). Замість single_mut() безпечніше використовувати get_single_mut()
    let Ok(mut transform) = query.single_mut() else { return };

    let speed = 25.0 * time.delta_secs();
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
        transform.translation += direction.normalize_or_zero() * speed;
    }
}