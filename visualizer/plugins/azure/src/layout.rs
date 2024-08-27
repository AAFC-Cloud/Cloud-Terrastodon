use avian2d::prelude::Collider;
use avian2d::prelude::DistanceJoint;
use avian2d::prelude::Joint;
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
        app.register_type::<DeactivatedCollider>();
        app.register_type::<LayoutJoint>();
    }
}

#[derive(Component, Debug, Reflect)]
pub struct LayoutJoint;

#[derive(Component, Debug, Reflect)]
pub struct DeactivatedCollider {
    #[reflect(ignore)]
    pub collider: Collider,
}

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
    subscription_query: Query<(Entity, Option<&Collider>), With<AzureSubscription>>,
    resource_group_query: Query<(Entity, Option<&Collider>), With<AzureResourceGroup>>,
) {
    let Ok(actions) = actions_query.get_single() else {
        warn!("Could not find actions");
        return;
    };
    if actions.just_pressed(&LayoutAction::Organize) {
        info!("Beginning organization");
        // Deactivate colliders
        for sub in subscription_query.iter().chain(resource_group_query.iter()) {
            let (sub_entity, sub_collider) = sub;
            if let Some(collider) = sub_collider {
                commands
                    .entity(sub_entity)
                    .remove::<Collider>()
                    .insert(DeactivatedCollider { collider: collider.clone() });
            }
        }
        // Create joints
        for [(s1, _), (s2, _)] in subscription_query.iter_combinations() {
            commands.spawn((
                LayoutJoint,
                Name::new("Layout Joint"),
                DistanceJoint::new(s1, s2)
                    .with_rest_length(2500.0)
                    .with_linear_velocity_damping(0.05)
                    .with_angular_velocity_damping(0.5)
                    .with_limits(800.0, 3000.0)
                    .with_compliance(0.000001),
            ));
        }
    }
}

fn end_organize(
    mut commands: Commands,
    actions_query: Query<&ActionState<LayoutAction>>,
    collider_fix_query: Query<(Entity, &DeactivatedCollider)>,
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
        for fix in collider_fix_query.iter() {
            let (entity, deactivated_collider) = fix;
            commands
                .entity(entity)
                .remove::<DeactivatedCollider>()
                .insert(deactivated_collider.collider.clone());
        }
    }
}

// from avian2d example for one way platform	
fn disable_collisions(
    mut one_way_platforms_query: Query<&mut OneWayPlatform>,
    other_colliders_query: Query<
        Option<&PassThroughOneWayPlatform>,
        (With<Collider>, Without<OneWayPlatform>), // NOTE: This precludes OneWayPlatform passing through a OneWayPlatform
    >,
    mut collisions: ResMut<Collisions>,
) {
    // This assumes that Collisions contains empty entries for entities
    // that were once colliding but no longer are.
    collisions.retain(|contacts| {
        // Differentiate between which normal of the manifold we should use
        enum RelevantNormal {
            Normal1,
            Normal2,
        }

        // First, figure out which entity is the one-way platform, and which is the other.
        // Choose the appropriate normal for pass-through depending on which is which.
        let (mut one_way_platform, other_entity, relevant_normal) =
            if let Ok(one_way_platform) = one_way_platforms_query.get_mut(contacts.entity1) {
                (one_way_platform, contacts.entity2, RelevantNormal::Normal1)
            } else if let Ok(one_way_platform) = one_way_platforms_query.get_mut(contacts.entity2) {
                (one_way_platform, contacts.entity1, RelevantNormal::Normal2)
            } else {
                // Neither is a one-way-platform, so accept the collision:
                // we're done here.
                return true;
            };

        if one_way_platform.0.contains(&other_entity) {
            let any_penetrating = contacts.manifolds.iter().any(|manifold| {
                manifold
                    .contacts
                    .iter()
                    .any(|contact| contact.penetration > 0.0)
            });

            if any_penetrating {
                // If we were already allowing a collision for a particular entity,
                // and if it is penetrating us still, continue to allow it to do so.
                return false;
            } else {
                // If it's no longer penetrating us, forget it.
                one_way_platform.0.remove(&other_entity);
            }
        }

        match other_colliders_query.get(other_entity) {
            // Pass-through is set to never, so accept the collision.
            Ok(Some(PassThroughOneWayPlatform::Never)) => true,
            // Pass-through is set to always, so always ignore this collision
            // and register it as an entity that's currently penetrating.
            Ok(Some(PassThroughOneWayPlatform::Always)) => {
                one_way_platform.0.insert(other_entity);
                false
            }
            // Default behaviour is "by normal".
            Err(_) | Ok(None) | Ok(Some(PassThroughOneWayPlatform::ByNormal)) => {
                // If all contact normals are in line with the local up vector of this platform,
                // then this collision should occur: the entity is on top of the platform.
                if contacts.manifolds.iter().all(|manifold| {
                    let normal = match relevant_normal {
                        RelevantNormal::Normal1 => manifold.normal1,
                        RelevantNormal::Normal2 => manifold.normal2,
                    };

                    normal.length() > Scalar::EPSILON && normal.dot(Vector::Y) >= 0.5
                }) {
                    true
                } else {
                    // Otherwise, ignore the collision and register
                    // the other entity as one that's currently penetrating.
                    one_way_platform.0.insert(other_entity);
                    false
                }
            }
        }
    });
}