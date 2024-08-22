use crate::resource_groups::AzureResourceGroup;
use crate::subscriptions::AzureSubscription;
use avian2d::math::Vector;
use avian2d::prelude::DistanceJoint;
use avian2d::prelude::Joint;
use bevy::color::palettes::css::RED;
use bevy::prelude::*;

pub struct JointsPlugin;
impl Plugin for JointsPlugin {
    fn build(&self, app: &mut App) {
        app.observe(on_resource_group_added);
        app.observe(on_subscription_added);
        app.add_systems(Update, draw_joints);
    }
}

fn on_resource_group_added(
    trigger: Trigger<OnAdd, AzureResourceGroup>,
    resource_group_query: Query<&AzureResourceGroup>,
    subscription_query: Query<(Entity, &AzureSubscription)>,
    mut commands: Commands,
) {
    let resource_group_entity = trigger.entity();
    let Ok(resource_group) = resource_group_query.get(resource_group_entity) else {
        warn!("Failed to find resource group {resource_group_entity:?}");
        return;
    };
    let subscription_id = &resource_group.subscription_id;
    // the same subscription can be represented by multiple entities in the world
    // for now lets just connect to all of them
    for (subscription_entity, subscription) in subscription_query.iter() {
        if subscription.subscription.id == *subscription_id {
            create_joint(&mut commands, subscription_entity, resource_group_entity);
        }
    }
}

fn on_subscription_added(
    trigger: Trigger<OnAdd, AzureSubscription>,
    subscription_query: Query<&AzureSubscription>,
    resource_group_query: Query<(Entity, &AzureResourceGroup)>,
    mut commands: Commands,
) {
    let subscription_entity = trigger.entity();
    let Ok(subscription) = subscription_query.get(subscription_entity) else {
        warn!("Failed to find subscription {subscription_entity:?}");
        return;
    };

    let subscription_id = &subscription.id;
    // let resource_group_entities =
    //     azure.get_resource_group_entities_for_subscription(subscription_id);
    for (resource_group_entity, resource_group) in resource_group_query.iter() {
        if resource_group.subscription_id == *subscription_id {
            create_joint(&mut commands, subscription_entity, resource_group_entity);
        }
    }
}

fn create_joint(commands: &mut Commands, subscription: Entity, resource_group: Entity) {
    let anchor = subscription;
    let object = resource_group;
    commands.spawn(
        DistanceJoint::new(anchor, object)
            .with_local_anchor_1(Vector::ZERO)
            .with_local_anchor_2(Vector::ZERO)
            .with_rest_length(1000.0)
            .with_linear_velocity_damping(0.1)
            .with_angular_velocity_damping(1.0)
            .with_compliance(0.00000001),
    );
}

fn draw_joints(
    mut gizmos: Gizmos,
    transform_query: Query<&Transform>,
    joint_query: Query<&DistanceJoint>,
    resource_group_query: Query<&AzureResourceGroup>,
    subscription_query: Query<&AzureSubscription>,
) {
    for joint in joint_query.iter() {
        let subscription_entity = joint.entity1;
        let resource_group_entity = joint.entity2;
        if !(resource_group_query.contains(resource_group_entity)
            && subscription_query.contains(subscription_entity))
        {
            continue;
        }
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
