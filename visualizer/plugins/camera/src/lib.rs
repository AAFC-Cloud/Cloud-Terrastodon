use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use leafwing_input_manager::prelude::InputManagerPlugin;
use leafwing_input_manager::prelude::InputMap;
use leafwing_input_manager::prelude::MouseScroll;
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
        app.add_systems(Update, handle_camera_input);
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Hash, Debug, Reflect)]
pub enum CameraAction {
    Wheel,
    Control,
    Shift,
}
impl Actionlike for CameraAction {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            CameraAction::Wheel => InputControlKind::DualAxis,
            CameraAction::Control => InputControlKind::Button,
            CameraAction::Shift => InputControlKind::Button,
        }
    }
}

fn setup(mut commands: Commands) {
    let input_map = InputMap::default()
        .with_dual_axis(CameraAction::Wheel, MouseScroll::default())
        .with(CameraAction::Control, KeyCode::ControlLeft)
        .with(CameraAction::Shift, KeyCode::ShiftLeft);
    commands
        .spawn(Camera2dBundle::default())
        .insert(InputManagerBundle::with_map(input_map));
}

fn handle_camera_input(
    mut query: Query<
        (
            &mut Transform,
            &mut OrthographicProjection,
            &ActionState<CameraAction>,
        ),
        With<Camera2d>,
    >,
) {
    const CAMERA_ZOOM_RATE: f32 = 0.05;
    const CAMERA_PAN_RATE: f32 = 10.;

    let camera = query.single_mut();
    let (mut camera_transform, mut camera_projection, action_state) = camera;

    let control = action_state.pressed(&CameraAction::Control);
    let shift = action_state.pressed(&CameraAction::Shift);
    let wheel: Vec2 = action_state.axis_pair(&CameraAction::Wheel);

    let (zoom_delta, pan): (f32, Vec2) = match (control, shift) {
        (true, _) => (wheel.y, Vec2::ZERO),
        (_, true) => (0., wheel.yx()),
        (false, false) => (0., wheel),
    };

    camera_projection.scale *= 1. - zoom_delta * CAMERA_ZOOM_RATE;
    camera_transform.translation += (pan * CAMERA_PAN_RATE).extend(0.);
}
