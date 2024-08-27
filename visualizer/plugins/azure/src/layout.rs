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

use crate::resource_groups::AzureResourceGroup;
use crate::subscriptions::AzureSubscription;

pub struct LayoutPlugin;
impl Plugin for LayoutPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<LayoutAction>::default());
        app.add_systems(Startup, setup);
        app.add_systems(Update, begin_organize);
        app.add_systems(Update, end_organize);
        app.add_systems(PostProcessCollisions, disable_collisions);
        app.register_type::<DisableCollisions>();
        app.register_type::<LayoutJoint>();
    }
}

#[derive(Component, Debug, Reflect)]
pub struct LayoutJoint;

#[derive(Component, Debug, Reflect)]
pub struct DisableCollisions;

#[derive(Eq, PartialEq, Clone, Copy, Hash, Debug, Reflect)]
pub enum LayoutAction {
    Organize,
}
impl Actionlike for LayoutAction {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            LayoutAction::Organize => InputControlKind::Button,
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Name::new("Layout Actions"),
        InputManagerBundle::with_map(
            InputMap::default().with(LayoutAction::Organize, KeyCode::Space),
        ),
    ));
}

fn begin_organize(
    mut commands: Commands,
    actions_query: Query<&ActionState<LayoutAction>>,
    subscription_query: Query<Entity, With<AzureSubscription>>,
    resource_group_query: Query<Entity, With<AzureResourceGroup>>,
) {
    let Ok(actions) = actions_query.get_single() else {
        warn!("Could not find actions");
        return;
    };
    if actions.just_pressed(&LayoutAction::Organize) {
        info!("Beginning organization");
        // Add DisableCollisions tag
        for sub in subscription_query.iter().chain(resource_group_query.iter()) {
            commands.entity(sub).insert(DisableCollisions);
        }
        // Create joints
        for [s1, s2] in subscription_query.iter_combinations() {
            commands.spawn((
                LayoutJoint,
                Name::new("Layout Joint"),
                DistanceJoint::new(s1, s2)
                    .with_rest_length(2500.0)
                    .with_linear_velocity_damping(0.05)
                    .with_angular_velocity_damping(0.5)
                    .with_limits(1200.0, 5000.0)
                    .with_compliance(0.000001),
            ));
        }
    }
}

fn end_organize(
    mut commands: Commands,
    actions_query: Query<&ActionState<LayoutAction>>,
    collider_fix_query: Query<Entity, With<DisableCollisions>>,
    joint_query: Query<Entity, With<LayoutJoint>>,
) {
    let Ok(actions) = actions_query.get_single() else {
        warn!("Could not find actions");
        return;
    };
    if actions.just_released(&LayoutAction::Organize) {
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
