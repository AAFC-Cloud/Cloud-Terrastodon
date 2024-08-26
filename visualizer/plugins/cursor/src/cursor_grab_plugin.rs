use avian2d::prelude::CollidingEntities;
use avian2d::prelude::LinearVelocity;
use avian2d::prelude::Position;
use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::Cursor;
use crate::CursorAction;

pub struct CursorGrabPlugin;

impl Plugin for CursorGrabPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Grabbed>();
        app.add_systems(Update, grab);
        app.add_systems(Update, attract);
    }
}

#[derive(Component, Debug, Reflect)]
pub struct Grabbed;

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
        grabbed_velocity.0 = error * 5.;
    }
}
