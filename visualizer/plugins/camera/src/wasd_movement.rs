use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use leafwing_input_manager::prelude::InputManagerPlugin;

use crate::camera_types::CameraAction;
use crate::camera_types::CameraMotion;
use crate::camera_types::PrimaryCamera;

// https://github.com/Leafwing-Studios/leafwing-input-manager/blob/9f9c3f3accac70f66e4160f00619add359d4311b/examples/mouse_wheel.rs

pub struct WasdCameraMovementPlugin;
impl Plugin for WasdCameraMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, wasd_movement);
        app.register_type::<CameraAction>();
        app.add_plugins(InputManagerPlugin::<CameraAction>::default());
    }
}
fn wasd_movement(
    mut query: Query<
        (
            &ActionState<CameraAction>,
            &mut CameraMotion,
            &mut LinearVelocity,
        ),
        With<PrimaryCamera>,
    >,
) {
    let Ok(camera) = query.get_single_mut() else {
        warn!("Camera not found");
        return;
    };
    let (action_state, mut camera_motion, mut camera_velocity) = camera;
    if action_state.just_pressed(&CameraAction::Sprint) {
        camera_motion.movement_speed = camera_motion.movement_speed_when_sprinting;
    } else if action_state.just_released(&CameraAction::Sprint) {
        camera_motion.movement_speed = camera_motion.movement_speed_default;
    }
    let direction: Vec2 = action_state.axis_pair(&CameraAction::Pan);
    camera_velocity.0 = direction * camera_motion.movement_speed;
}
