use avian2d::math::Vector;
use avian2d::prelude::DistanceJoint;
use avian2d::prelude::Joint;
use bevy::color::palettes::css::RED;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;

pub struct JointsPlugin;
impl Plugin for JointsPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<MyJointGizmos>();
        app.add_systems(Startup, configure_gizmos);
        app.add_systems(Update, draw_joints);
    }
}

// We can create our own gizmo config group!
#[derive(Default, Reflect, GizmoConfigGroup)]
struct MyJointGizmos {}

#[derive(Component, Reflect, Debug)]
pub struct DrawThisJoint;

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
                create_joint(commands, added_entity, existing_entity)
            },
            AddedRole::AddedFollower => |commands, added_entity, existing_entity| {
                create_joint(commands, existing_entity, added_entity)
            },
        };
        for (existing_entity, existing) in existing_query.iter() {
            if matcher(added, existing) {
                make_joint(&mut commands, added_entity, existing_entity);
            }
        }
    }
}

fn create_joint(commands: &mut Commands, leader: Entity, follower: Entity) {
    commands.spawn((
        Name::new("Drawable Joint"),
        DrawThisJoint,
        DistanceJoint::new(leader, follower)
            .with_local_anchor_1(Vector::ZERO)
            .with_local_anchor_2(Vector::ZERO)
            .with_rest_length(500.0)
            .with_linear_velocity_damping(0.05) // Reduced damping for more springiness
            .with_angular_velocity_damping(0.5) // Reduced angular damping
            .with_compliance(0.00001), // Increased compliance for more flexibility
    ));
}

fn draw_joints(
    mut gizmos: Gizmos<MyJointGizmos>,
    transform_query: Query<&Transform>,
    joint_query: Query<&DistanceJoint, With<DrawThisJoint>>,
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
