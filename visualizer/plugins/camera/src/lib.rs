use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use leafwing_input_manager::prelude::InputManagerPlugin;
use leafwing_input_manager::prelude::InputMap;
use leafwing_input_manager::prelude::KeyboardVirtualDPad;
use leafwing_input_manager::prelude::MouseScrollAxis;
use leafwing_input_manager::Actionlike;
use leafwing_input_manager::InputControlKind;
use leafwing_input_manager::InputManagerBundle;

// https://github.com/Leafwing-Studios/leafwing-input-manager/blob/9f9c3f3accac70f66e4160f00619add359d4311b/examples/mouse_wheel.rs
pub struct MyCameraPlugin;

impl Plugin for MyCameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CameraAction>();
        app.add_plugins(InputManagerPlugin::<CameraAction>::default());
        app.add_systems(Startup, setup);
        app.add_systems(Update, zoom_camera);
        app.add_systems(Update, pan_camera);
    }
}

#[derive(Component, Debug)]
pub struct CameraMotion {
    zoom_speed: f32,
    zoom_speed_default: f32,
    zoom_speed_when_sprinting: f32,
    movement_speed: f32,
    movement_speed_default: f32,
    movement_speed_when_sprinting: f32,
}
impl Default for CameraMotion {
    fn default() -> Self {
        Self {
            zoom_speed: 0.05,
            zoom_speed_default: 0.05,
            zoom_speed_when_sprinting: 0.2,
            movement_speed: 10.,
            movement_speed_default: 10.,
            movement_speed_when_sprinting: 50.,
        }
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Hash, Debug, Reflect)]
pub enum CameraAction {
    Zoom,
    Pan,
    Sprint,
}
impl Actionlike for CameraAction {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            CameraAction::Zoom => InputControlKind::Axis,
            CameraAction::Pan => InputControlKind::DualAxis,
            CameraAction::Sprint => InputControlKind::Button,
        }
    }
}

fn setup(mut commands: Commands) {
    let input_map = InputMap::default()
        .with_axis(CameraAction::Zoom, MouseScrollAxis::Y)
        .with_dual_axis(CameraAction::Pan, KeyboardVirtualDPad::WASD)
        .with(CameraAction::Sprint, KeyCode::ShiftLeft);
    commands
        .spawn((Camera2dBundle::default(), CameraMotion::default()))
        .insert(InputManagerBundle::with_map(input_map));
}

fn zoom_camera(
    mut query: Query<
        (
            &mut OrthographicProjection,
            &ActionState<CameraAction>,
            &mut CameraMotion,
        ),
        With<Camera2d>,
    >,
) {
    let camera = query.single_mut();
    let (mut camera_projection, action_state, mut camera_motion) = camera;
    if action_state.just_pressed(&CameraAction::Sprint) {
        camera_motion.zoom_speed = camera_motion.zoom_speed_when_sprinting;
    } else if action_state.just_released(&CameraAction::Sprint) {
        camera_motion.zoom_speed = camera_motion.zoom_speed_default;
    }
    let zoom_delta = action_state.value(&CameraAction::Zoom);
    camera_projection.scale *= 1. - zoom_delta * camera_motion.zoom_speed;
}

fn pan_camera(
    mut query: Query<
        (
            &mut Transform,
            &ActionState<CameraAction>,
            &mut CameraMotion,
        ),
        With<Camera2d>,
    >,
) {
    let camera = query.single_mut();
    let (mut camera_transform, action_state, mut camera_motion) = camera;
    if action_state.just_pressed(&CameraAction::Sprint) {
        camera_motion.movement_speed = camera_motion.movement_speed_when_sprinting;
    } else if action_state.just_released(&CameraAction::Sprint) {
        camera_motion.movement_speed = camera_motion.movement_speed_default;
    }
    let dir: Vec2 = action_state.axis_pair(&CameraAction::Pan);
    camera_transform.translation += dir.extend(0.) * camera_motion.movement_speed;
}
