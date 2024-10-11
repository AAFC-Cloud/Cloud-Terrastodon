mod cursor_grab_plugin;
mod cursor_hover_plugin;

use avian2d::prelude::Collider;
use avian2d::prelude::CollisionLayers;
use avian2d::prelude::LayerMask;
use avian2d::prelude::Position;
use avian2d::prelude::RigidBody;
use avian2d::prelude::Sensor;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use cloud_terrastodon_visualizer_camera_plugin::PrimaryCamera;
use cloud_terrastodon_visualizer_physics_plugin::prelude::PhysLayer;
use cursor_grab_plugin::CursorGrabPlugin;
use cursor_hover_plugin::CursorHoverPlugin;
use leafwing_input_manager::prelude::InputManagerPlugin;
use leafwing_input_manager::prelude::InputMap;
use leafwing_input_manager::Actionlike;
use leafwing_input_manager::InputControlKind;
use leafwing_input_manager::InputManagerBundle;

pub mod prelude {
    pub use crate::cursor_grab_plugin::*;
    pub use crate::cursor_hover_plugin::*;
    pub use crate::CursorPlugin;
    pub use crate::CursorAction;
}

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CursorHoverPlugin);
        app.add_plugins(CursorGrabPlugin);
        app.register_type::<Cursor>();
        app.add_plugins(InputManagerPlugin::<CursorAction>::default());
        app.add_systems(Startup, setup);
        app.add_systems(Update, update_cursor);
    }
}

#[derive(Component, Debug, Reflect)]
pub struct Cursor;

#[derive(Eq, PartialEq, Clone, Copy, Hash, Debug, Reflect)]
pub enum CursorAction {
    Grab,
    ToggleAltInfo,
}
impl Actionlike for CursorAction {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            CursorAction::Grab => InputControlKind::Button,
            CursorAction::ToggleAltInfo => InputControlKind::Button,
        }
    }
}

fn setup(mut commands: Commands) {
    let input_map = InputMap::default()
        .with(CursorAction::Grab, MouseButton::Left)
        .with(CursorAction::ToggleAltInfo, KeyCode::AltLeft);

    commands
        .spawn((
            Cursor,
            RigidBody::Kinematic,
            Collider::circle(25.0),
            Sensor,
            Name::new("Cursor"),
            SpatialBundle::default(),
            CollisionLayers::new(PhysLayer::Cursor, LayerMask::ALL)
        ))
        .insert(InputManagerBundle::with_map(input_map));
}

fn update_cursor(
    mut cursor_query: Query<&mut Position, With<Cursor>>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<PrimaryCamera>>,
) {
    let Ok((camera, camera_transform)) = camera.get_single() else {
        warn!("There wasn't exactly 1 camera found!");
        return;
    };
    let Ok(window) = window.get_single() else {
        warn!("There wasn't exactly 1 window found!");
        return;
    };
    let Some(viewport_position) = window.cursor_position() else {
        // cursor outside the window, do nothing
        return;
    };
    let Some(world_position) = camera.viewport_to_world_2d(camera_transform, viewport_position)
    else {
        return;
    };
    for cursor in cursor_query.iter_mut() {
        let mut cursor_position = cursor;
        cursor_position.0 = world_position;
    }
}
