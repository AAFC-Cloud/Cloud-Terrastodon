use avian2d::prelude::Collisions;
use avian2d::prelude::DistanceJoint;
use avian2d::prelude::Joint;
use avian2d::prelude::PostProcessCollisions;
use bevy::prelude::*;

use leafwing_input_manager::prelude::ActionState;
use leafwing_input_manager::prelude::InputManagerPlugin;
use leafwing_input_manager::prelude::InputMap;
use leafwing_input_manager::Actionlike;
use leafwing_input_manager::InputControlKind;
use leafwing_input_manager::InputManagerBundle;

pub struct OrganizerPlugin;
impl Plugin for OrganizerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<OrganizerAction>::default());
        app.add_systems(Startup, setup);
        app.add_systems(Update, begin_organize);
        app.add_systems(Update, end_organize);
        app.add_systems(PostProcessCollisions, disable_collisions);
        app.register_type::<DisableCollisions>();
        app.register_type::<OrganizationJoint>();
    }
}

#[derive(Component, Debug, Reflect)]
pub struct OrganizablePrimary;

#[derive(Component, Debug, Reflect)]
pub struct OrganizableSecondary;

#[derive(Component, Debug, Reflect)]
pub struct OrganizationJoint;

#[derive(Component, Debug, Reflect)]
pub struct DisableCollisions;

#[derive(Eq, PartialEq, Clone, Copy, Hash, Debug, Reflect)]
pub enum OrganizerAction {
    Organize,
}
impl Actionlike for OrganizerAction {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            OrganizerAction::Organize => InputControlKind::Button,
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Name::new("Layout Actions"),
        InputManagerBundle::with_map(
            InputMap::default().with(OrganizerAction::Organize, KeyCode::Space),
        ),
    ));
}

fn begin_organize(
    mut commands: Commands,
    actions_query: Query<&ActionState<OrganizerAction>>,
    primary_query: Query<Entity, With<OrganizablePrimary>>,
    secondary_query: Query<Entity, With<OrganizableSecondary>>,
) {
    let Ok(actions) = actions_query.get_single() else {
        warn!("Could not find actions");
        return;
    };
    if actions.just_pressed(&OrganizerAction::Organize) {
        info!("Beginning organization");
        // Add DisableCollisions tag
        for primary in primary_query.iter().chain(secondary_query.iter()) {
            commands.entity(primary).insert(DisableCollisions);
        }
        // Create joints
        for [e1, e2] in primary_query.iter_combinations() {
            commands.spawn((
                OrganizationJoint,
                Name::new("Organization Joint"),
                DistanceJoint::new(e1, e2)
                    .with_rest_length(4000.0)
                    .with_linear_velocity_damping(0.05)
                    .with_angular_velocity_damping(0.5)
                    .with_limits(2500.0, 8000.0)
                    .with_compliance(0.000001),
            ));
        }
    }
}

fn end_organize(
    mut commands: Commands,
    actions_query: Query<&ActionState<OrganizerAction>>,
    collider_fix_query: Query<Entity, With<DisableCollisions>>,
    joint_query: Query<Entity, With<OrganizationJoint>>,
) {
    let Ok(actions) = actions_query.get_single() else {
        warn!("Could not find actions");
        return;
    };
    if actions.just_released(&OrganizerAction::Organize) {
        info!("Ending organization");
        for joint in joint_query.iter() {
            commands.entity(joint).despawn();
        }
        for entity in collider_fix_query.iter() {
            commands.entity(entity).remove::<DisableCollisions>();
        }
    }
}
fn disable_collisions(
    disable_collisions_query: Query<Entity, With<DisableCollisions>>,
    mut collisions: ResMut<Collisions>,
) {
    collisions.retain(|contacts| {
        // If either entity has the DisableCollisions tag, prevent the collision
        if disable_collisions_query.get(contacts.entity1).is_ok()
            || disable_collisions_query.get(contacts.entity2).is_ok()
        {
            return false;
        }
        // Otherwise, allow the collision to occur
        true
    });
}
