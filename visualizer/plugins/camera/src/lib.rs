mod drag_pan;
mod scroll_zoom;
mod wasd_movement;
mod spawn_camera_plugin;
mod camera_types;


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
use camera_types::CameraTypesPlugin;
use cloud_terrastodon_visualizer_physics_plugin::prelude::CustomLinearDamping;
use leafwing_input_manager::prelude::ActionState;
use leafwing_input_manager::prelude::InputManagerPlugin;
use leafwing_input_manager::prelude::InputMap;
use leafwing_input_manager::prelude::KeyboardVirtualDPad;
use leafwing_input_manager::prelude::MouseScrollAxis;
use leafwing_input_manager::Actionlike;
use leafwing_input_manager::InputControlKind;
use leafwing_input_manager::InputManagerBundle;
use scroll_zoom::ScrollZoomCameraPlugin;
use spawn_camera_plugin::SpawnCameraPlugin;
use wasd_movement::WasdCameraMovementPlugin;

pub struct CameraLibPlugin;

impl Plugin for CameraLibPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CameraTypesPlugin);
        app.add_plugins(SpawnCameraPlugin);
        app.add_plugins(ScrollZoomCameraPlugin);
        app.add_plugins(WasdCameraMovementPlugin);
    }
}

pub mod prelude {
    pub use crate::camera_types::*;
}