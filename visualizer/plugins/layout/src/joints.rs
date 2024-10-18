use avian2d::prelude::DistanceJoint;
use avian2d::prelude::Joint;
use bevy::color::palettes::css::RED;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy::utils::hashbrown::HashSet;
use core::f32;
use itertools::Itertools;

pub struct JointsPlugin;
impl Plugin for JointsPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<MyJointGizmos>();
        app.add_systems(Startup, configure_gizmos);
        app.add_systems(Update, draw_joints);
        app.observe(on_leader_follower_joint_added);
    }
}

// We can create our own gizmo config group!
#[derive(Default, Reflect, GizmoConfigGroup)]
struct MyJointGizmos {}

#[derive(Component, Reflect, Debug)]
pub struct LeaderFollowerJoint;
#[derive(Component, Reflect, Debug)]
pub struct FollowerFollowerJoint;

fn configure_gizmos(mut config_store: ResMut<GizmoConfigStore>) {
    // Gizmos and sprites: How can I draw a SpriteBundle on top of a Gizmo?
    // https://github.com/bevyengine/bevy/discussions/11601

    // Gizmos always draw on 2d sprites, z value and depth_bias neither works
    // https://github.com/bevyengine/bevy/issues/13027

    let (config, _) = config_store.config_mut::<MyJointGizmos>();
    config.render_layers = RenderLayers::layer(1);
}

pub fn join_on_leader_added<THIS, OTHER, MATCHER>(
    matcher: MATCHER,
) -> impl FnMut(Trigger<OnAdd, THIS>, Query<&THIS>, Query<(Entity, &OTHER)>, Commands)
where
    THIS: Component + Sized,
    OTHER: Component + Sized,
    MATCHER: Fn(&THIS, &OTHER) -> bool,
{
    join_on_thing_added(matcher, AddedRole::AddedLeader)
}

pub fn join_on_follower_added<THIS, OTHER, MATCHER>(
    matcher: MATCHER,
) -> impl FnMut(Trigger<OnAdd, THIS>, Query<&THIS>, Query<(Entity, &OTHER)>, Commands)
where
    THIS: Component + Sized,
    OTHER: Component + Sized,
    MATCHER: Fn(&THIS, &OTHER) -> bool,
{
    join_on_thing_added(matcher, AddedRole::AddedFollower)
}

pub enum AddedRole {
    AddedLeader,
    AddedFollower,
}

pub fn join_on_thing_added<THIS, OTHER, MATCHER>(
    matcher: MATCHER,
    role: AddedRole,
) -> impl FnMut(Trigger<OnAdd, THIS>, Query<&THIS>, Query<(Entity, &OTHER)>, Commands)
where
    THIS: Component + Sized,
    OTHER: Component + Sized,
    MATCHER: Fn(&THIS, &OTHER) -> bool,
{
    move |trigger: Trigger<OnAdd, THIS>,
          added_query: Query<&THIS>,
          existing_query: Query<(Entity, &OTHER)>,
          mut commands: Commands| {
        let added_entity = trigger.entity();
        let Ok(added) = added_query.get(added_entity) else {
            warn!(
                "Failed to find this {} {added_entity:?}",
                std::any::type_name::<THIS>()
            );
            return;
        };
        let make_joint: fn(&mut Commands, Entity, Entity) = match role {
            AddedRole::AddedLeader => |commands, added_entity, existing_entity| {
                create_leader_follower_joint(commands, added_entity, existing_entity)
            },
            AddedRole::AddedFollower => |commands, added_entity, existing_entity| {
                create_leader_follower_joint(commands, existing_entity, added_entity)
            },
        };
        for (existing_entity, existing) in existing_query.iter() {
            if matcher(added, existing) {
                make_joint(&mut commands, added_entity, existing_entity);
            }
        }
    }
}

fn create_leader_follower_joint(commands: &mut Commands, leader: Entity, follower: Entity) {
    commands.spawn((
        Name::new("Leader-Follower Joint"),
        LeaderFollowerJoint,
        DistanceJoint::new(leader, follower)
            .with_rest_length(2000.0)
            .with_limits(500.0, 8000.0)
            .with_linear_velocity_damping(0.05)
            .with_angular_velocity_damping(0.5)
            .with_compliance(0.00001),
    ));
}

fn create_follower_follower_joint(commands: &mut Commands, follower1: Entity, follower2: Entity) {
    commands.spawn((
        Name::new("Follower-Follower Joint"),
        FollowerFollowerJoint,
        DistanceJoint::new(follower1, follower2)
            .with_limits(500.0, 10000.0)
            .with_rest_length(2000.0)
            .with_linear_velocity_damping(0.05)
            .with_angular_velocity_damping(0.5)
            .with_compliance(0.00001),
    ));
}

fn on_leader_follower_joint_added(
    trigger: Trigger<OnAdd, LeaderFollowerJoint>,
    lf_joint_query: Query<
        &DistanceJoint,
        (With<LeaderFollowerJoint>, Without<FollowerFollowerJoint>),
    >,
    ff_joint_query: Query<
        &DistanceJoint,
        (With<FollowerFollowerJoint>, Without<LeaderFollowerJoint>),
    >,
    mut commands: Commands,
) {
    let added_entity = trigger.entity();
    let Ok(added_joint) = lf_joint_query.get(added_entity) else {
        warn!(
            "Failed to find new {} {added_entity:?}",
            std::any::type_name::<DistanceJoint>()
        );
        return;
    };

    // We want to find the other followers of this leader
    let leader_entity = added_joint.entity1;
    let mut followers = HashSet::new();
    for joint in lf_joint_query.iter() {
        if joint.entity1 == leader_entity {
            let follower_entity = joint.entity2;
            followers.insert(follower_entity);
        }
    }

    let mut existing_ff_joints = HashSet::new();
    for joint in ff_joint_query.iter() {
        if followers.contains(&joint.entity1) || followers.contains(&joint.entity2) {
            existing_ff_joints.insert((joint.entity1, joint.entity2));
        }
    }

    // We want to create joints between the follower pairs where no joint exists
    let need_ff_joint = followers
        .into_iter()
        .permutations(2)
        .map(|x| (x[0], x[1]))
        .filter(|(a, b)| a != b)
        .filter(|(a, b)| {
            !existing_ff_joints.contains(&(*a, *b)) && !existing_ff_joints.contains(&(*b, *a))
        })
        .dedup_by(|a, b| a == b || a == &(b.1, b.0));
    for pair in need_ff_joint {
        create_follower_follower_joint(&mut commands, pair.0, pair.1);
    }
}

fn draw_joints(
    mut gizmos: Gizmos<MyJointGizmos>,
    transform_query: Query<&Transform>,
    joint_query: Query<&DistanceJoint, With<LeaderFollowerJoint>>,
) {
    for joint in joint_query.iter() {
        let subscription_entity = joint.entity1;
        let resource_group_entity = joint.entity2;
        let Ok([subscription_transform, resource_group_transform]) =
            transform_query.get_many([subscription_entity, resource_group_entity])
        else {
            warn!("Couldn't find transform for (Subscription={subscription_entity:?},ResourceGroup={resource_group_entity:?}");
            continue;
        };
        gizmos.line_2d(
            subscription_transform.translation.xy(),
            resource_group_transform.translation.xy(),
            RED,
        );
    }
}
