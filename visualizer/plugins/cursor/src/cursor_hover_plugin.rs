use avian2d::prelude::CollidingEntities;
use bevy::prelude::*;
use std::collections::HashSet;

use crate::Cursor;

pub struct CursorHoverPlugin;

impl Plugin for CursorHoverPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Hovered>();
        app.register_type::<DirectHovered>();
        app.add_systems(Update, hover);
        app.add_systems(Update, unhover);
        app.observe(hover_insert_propagate);
        app.observe(hover_remove_propagate);
        app.observe(show_when_hovered);
        app.observe(hide_when_not_hovered);
        app.observe(hide_when_not_hovered_init);
    }
}


#[derive(Component, Debug, Reflect)]
pub struct DirectHovered;
#[derive(Component, Debug, Reflect)]
pub struct Hovered;

#[derive(Component, Debug, Reflect)]
pub struct OnlyShowWhenHovered;

fn hover(
    mut commands: Commands,
    cursor_query: Query<&CollidingEntities, With<Cursor>>,
    query: Query<Entity, (Without<Cursor>, Without<DirectHovered>)>,
) {
    for cursor in cursor_query.iter() {
        let cursor_hits = cursor;
        for hit in cursor_hits.iter() {
            if query.get(*hit).is_ok() {
                commands.entity(*hit).insert(Hovered);
                commands.entity(*hit).insert(DirectHovered);
                debug!("Now hovering {hit}");
            }
        }
    }
}

fn unhover(
    mut commands: Commands,
    cursor_query: Query<&CollidingEntities, With<Cursor>>,
    hovered_query: Query<Entity, (With<DirectHovered>, Without<Cursor>)>,
) {
    let mut hovering = HashSet::new();
    for cursor in cursor_query.iter() {
        let cursor_hits = cursor;
        for hit in cursor_hits.iter() {
            hovering.insert(hit);
        }
    }
    for hovered in hovered_query.iter() {
        if !hovering.contains(&hovered) {
            commands.entity(hovered).remove::<Hovered>();
            commands.entity(hovered).remove::<DirectHovered>();
            debug!("No longer hovering {hovered}");
        }
    }
}

fn hover_insert_propagate(
    trigger: Trigger<OnInsert, Hovered>,
    mut commands: Commands,
    query: Query<&Children>,
) {
    let Ok(thing) = query.get(trigger.entity()) else {
        warn!("Could not find entity {} for system hover_insert_propagate", trigger.entity());
        return;
    };
    let children = thing;
    for child in children {	
        commands.entity(*child).insert(Hovered);
        info!("Propagated hover to {child}");
    }
}

fn hover_remove_propagate(
    trigger: Trigger<OnRemove, Hovered>,
    mut commands: Commands,
    query: Query<&Children>,
) {
    let Ok(thing) = query.get(trigger.entity()) else {
        warn!("Could not find entity {} for system hover_remove_propagate", trigger.entity());
        return;
    };
    let children = thing;
    for child in children {
        commands.entity(*child).remove::<Hovered>();
        info!("Propagated unhover to {child}");
    }
}

fn show_when_hovered(
    trigger: Trigger<OnAdd, Hovered>,
    mut query: Query<&mut Visibility, With<OnlyShowWhenHovered>>,
) {
    let Ok(thing) = query.get_mut(trigger.entity()) else {
        warn!("Could not find entity {} for system show_when_hovered", trigger.entity());
        return;
    };
    let mut visibility = thing;
    *visibility = Visibility::Inherited;
    info!("Set {} to {:?}",trigger.entity(), visibility);
}

fn hide_when_not_hovered(
    trigger: Trigger<OnRemove, Hovered>,
    mut query: Query<&mut Visibility, With<OnlyShowWhenHovered>>,
) {
    let Ok(thing) = query.get_mut(trigger.entity()) else {
        warn!("Could not find entity {} for system hide_when_not_hovered", trigger.entity());
        return;
    };
    let mut visibility = thing;
    *visibility = Visibility::Hidden;
    info!("Set {} to {:?}",trigger.entity(), visibility);
}

fn hide_when_not_hovered_init(
    trigger: Trigger<OnAdd, OnlyShowWhenHovered>,
    mut query: Query<&mut Visibility>,
) {
    let Ok(thing) = query.get_mut(trigger.entity()) else {
        warn!("Could not find entity {} for system hide_when_not_hovered_init", trigger.entity());
        return;
    };
    let mut visibility = thing;
    *visibility = Visibility::Hidden;
    info!("Set initial {} to {:?}",trigger.entity(), visibility);
}