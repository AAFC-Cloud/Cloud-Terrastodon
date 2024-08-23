use avian2d::prelude::Collider;
use avian2d::prelude::CollidingEntities;
use avian2d::prelude::LinearVelocity;
use avian2d::prelude::Position;
use avian2d::prelude::RigidBody;
use avian2d::prelude::Sensor;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use leafwing_input_manager::prelude::ActionState;
use leafwing_input_manager::prelude::InputManagerPlugin;
use leafwing_input_manager::prelude::InputMap;
use leafwing_input_manager::Actionlike;
use leafwing_input_manager::InputControlKind;
use leafwing_input_manager::InputManagerBundle;

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Cursor>();
        app.register_type::<Grabbed>();
        app.add_plugins(InputManagerPlugin::<CursorAction>::default());
        app.add_systems(Startup, setup);
        app.add_systems(Update, update_cursor);
        app.add_systems(Update, grab);
        app.add_systems(Update, attract);
    }
}

#[derive(Component, Debug, Reflect)]
pub struct Cursor;

#[derive(Component, Debug, Reflect)]
pub struct Grabbed;

#[derive(Eq, PartialEq, Clone, Copy, Hash, Debug, Reflect)]
pub enum CursorAction {
    Grab,
}
impl Actionlike for CursorAction {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            CursorAction::Grab => InputControlKind::Button,
        }
    }
}

fn setup(mut commands: Commands) {
    let input_map = InputMap::default().with(CursorAction::Grab, MouseButton::Left);

    commands
        .spawn((
            Cursor,
            RigidBody::Kinematic,
            Collider::circle(1.0),
            Sensor,
            Name::new("Cursor"),
            SpatialBundle::default(),
        ))
        .insert(InputManagerBundle::with_map(input_map));
}

fn update_cursor(
    mut cursor_query: Query<&mut Position, With<Cursor>>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
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

fn grab(
    mut commands: Commands,
    cursor_query: Query<(&ActionState<CursorAction>, &CollidingEntities), With<Cursor>>,
    grabbed_query: Query<Entity, (Without<Cursor>, With<Grabbed>)>,
) {
    for cursor in cursor_query.iter() {
        let (cursor_actions, cursor_hits) = cursor;
        if cursor_actions.just_pressed(&CursorAction::Grab) {
            for hit in cursor_hits.iter() {
                commands.entity(*hit).insert(Grabbed);
                debug!("Now grabbing {hit}");
            }
        } else if cursor_actions.just_released(&CursorAction::Grab) {
            for grabbed in grabbed_query.iter() {
                commands.entity(grabbed).remove::<Grabbed>();
                debug!("No longer grabbing {grabbed}");
            }
        }
    }
}

fn attract(
    cursor_query: Query<&Position, With<Cursor>>,
    mut grabbed_query: Query<(&mut LinearVelocity, &Position), With<Grabbed>>,
) {
    let Ok(cursor) = cursor_query.get_single() else {
        warn!("Failed to find exactly 1 cursor");
        return;
    };
    let cursor_position = cursor;
    for grabbed in grabbed_query.iter_mut() {
        let (mut grabbed_velocity, grabbed_position) = grabbed;        
        // attract towards cursor position
        let error = cursor_position.0 - grabbed_position.0;
        grabbed_velocity.0 = error * 2.;
    }
}