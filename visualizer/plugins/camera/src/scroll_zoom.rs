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
use cloud_terrastodon_visualizer_physics_plugin::prelude::CustomLinearDamping;
use leafwing_input_manager::prelude::ActionState;
use leafwing_input_manager::prelude::InputManagerPlugin;
use leafwing_input_manager::prelude::InputMap;
use leafwing_input_manager::prelude::KeyboardVirtualDPad;
use leafwing_input_manager::prelude::MouseScrollAxis;
use leafwing_input_manager::Actionlike;
use leafwing_input_manager::InputControlKind;
use leafwing_input_manager::InputManagerBundle;

use bevy::prelude::Plugin;

use crate::camera_types::CameraAction;
use crate::camera_types::CameraMotion;
use crate::camera_types::PrimaryCamera;

pub struct ScrollZoomCameraPlugin;
impl Plugin for ScrollZoomCameraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, scroll_zoom);
    }
}
fn scroll_zoom(
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
