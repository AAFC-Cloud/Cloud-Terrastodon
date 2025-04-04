use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use leafwing_input_manager::Actionlike;
use leafwing_input_manager::InputControlKind;

pub struct CameraTypesPlugin;
impl Plugin for CameraTypesPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PrimaryCamera>();
        app.register_type::<JointGizmosCamera>();
        app.register_type::<CameraMotion>();
        app.register_type::<CameraAction>();
    }
}

#[derive(Component, Debug, Reflect)]
pub struct PrimaryCamera;
#[derive(Component, Debug, Reflect)]
pub struct JointGizmosCamera;

#[derive(Component, Debug, Reflect)]
pub struct CameraMotion {
    pub zoom_speed: f32,
    pub zoom_speed_default: f32,
    pub zoom_speed_when_sprinting: f32,
    pub movement_speed: f32,
    pub movement_speed_default: f32,
    pub movement_speed_when_sprinting: f32,
}
impl Default for CameraMotion {
    fn default() -> Self {
        Self {
            zoom_speed: 0.05,
            zoom_speed_default: 0.05,
            zoom_speed_when_sprinting: 0.2,
            movement_speed: 1000.,
            movement_speed_default: 250.,
            movement_speed_when_sprinting: 10000.,
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
