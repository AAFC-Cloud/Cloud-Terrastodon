mod drag_pan;
mod scroll_zoom;
mod wasd_movement;
mod spawn_camera_plugin;
mod camera_types;


use bevy::prelude::*;
use camera_types::CameraTypesPlugin;
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