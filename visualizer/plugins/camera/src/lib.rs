use avian2d::prelude::Collider;
use avian2d::prelude::FixedJoint;
use avian2d::prelude::Joint;
use avian2d::prelude::LinearVelocity;
use avian2d::prelude::MassPropertiesBundle;
use avian2d::prelude::RigidBody;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy::window::PrimaryWindow;
use bevy_egui::EguiContext;
use cloud_terrastodon_visualizer_damping_plugin::CustomLinearDamping;
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
pub struct PrimaryCamera;
#[derive(Component, Debug)]
pub struct JointGizmosCamera;

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
            movement_speed: 250.,
            movement_speed_default: 250.,
            movement_speed_when_sprinting: 5000.,
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

pub enum MyRenderLayers {
    Primary,
    JointGizmos,
}
impl MyRenderLayers {
    pub fn layer(self) -> RenderLayers {
        self.into()
    }
}
impl From<MyRenderLayers> for RenderLayers {
    fn from(value: MyRenderLayers) -> Self {
        match value {
            MyRenderLayers::Primary => RenderLayers::layer(1),
            MyRenderLayers::JointGizmos => RenderLayers::layer(0),
        }
    }
}

fn setup(mut commands: Commands) {
    let input_map = InputMap::default()
        .with_axis(CameraAction::Zoom, MouseScrollAxis::Y)
        .with_dual_axis(CameraAction::Pan, KeyboardVirtualDPad::WASD)
        .with(CameraAction::Sprint, KeyCode::ShiftLeft);

    let primary_camera_id = commands
        .spawn((
            Camera2dBundle {
                camera: Camera {
                    order: 0,
                    ..default()
                },
                ..default()
            },
            CameraMotion::default(),
            RigidBody::Kinematic,
            LinearVelocity::default(),
            CustomLinearDamping::default(),
            PrimaryCamera,
            MyRenderLayers::Primary.layer(),
            Name::new("Primary Camera"),
        ))
        .insert(InputManagerBundle::with_map(input_map))
        .id();

    let joint_gizmos_camera_id = commands
        .spawn((
            Camera2dBundle {
                camera: Camera {
                    order: 1,
                    ..default()
                },
                ..default()
            },
            JointGizmosCamera,
            MyRenderLayers::JointGizmos.layer(),
            RigidBody::Dynamic,
            MassPropertiesBundle::new_computed(&Collider::rectangle(1., 1.), 1.0),
            Name::new("Joint Gizmos Camera"),
        ))
        .id();

    commands.spawn(FixedJoint::new(primary_camera_id, joint_gizmos_camera_id));
}

fn zoom_camera(
    mut primary_camera_query: Query<
        (
            &mut OrthographicProjection,
            &ActionState<CameraAction>,
            &mut CameraMotion,
        ),
        With<PrimaryCamera>,
    >,
    mut other_camera_query: Query<
        &mut OrthographicProjection,
        (With<Camera>, Without<PrimaryCamera>),
    >,
    egui_context_query: Query<&EguiContext, With<PrimaryWindow>>,
) {
    let egui_wants_pointer = egui_context_query
        .get_single()
        .ok()
        .map(|ctx| {
            let mut ctx = ctx.clone();
            let ctx = ctx.get_mut();
            ctx.is_using_pointer() || ctx.is_pointer_over_area()
        })
        .unwrap_or(false);
    if egui_wants_pointer {
        return;
    }

    // update primary camera
    let camera = primary_camera_query.single_mut();
    let (mut primary_camera_projection, action_state, mut camera_motion) = camera;
    if action_state.just_pressed(&CameraAction::Sprint) {
        camera_motion.zoom_speed = camera_motion.zoom_speed_when_sprinting;
    } else if action_state.just_released(&CameraAction::Sprint) {
        camera_motion.zoom_speed = camera_motion.zoom_speed_default;
    }
    let zoom_delta = action_state.value(&CameraAction::Zoom);
    primary_camera_projection.scale *= 1. - zoom_delta * camera_motion.zoom_speed;

    // update other cameras to match
    for mut other_camera_projection in other_camera_query.iter_mut() {
        other_camera_projection.scale = primary_camera_projection.scale;
    }
}

fn pan_camera(
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
