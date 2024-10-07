use avian2d::prelude::DistanceJoint;
use avian2d::prelude::Joint;
use avian2d::prelude::RigidBody;
use avian2d::prelude::Sensor;
use bevy::prelude::*;

pub struct BiasPlugin;

impl Plugin for BiasPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<BiasTowardsOrigin>();
        app.register_type::<OriginAnchor>();
        app.add_systems(Startup, spawn_anchor);
        app.observe(apply_bias);
    }
}

#[derive(Component, Reflect, Debug)]
pub struct BiasTowardsOrigin;

#[derive(Component, Reflect, Debug)]
pub struct OriginAnchor;

fn spawn_anchor(mut commands: Commands) {
    commands.spawn((
        Name::new("Origin anchor"),
        SpatialBundle {
            transform: Transform::from_translation(Vec3::ZERO),
            ..default()
        },
        RigidBody::Static,
        Sensor,
        OriginAnchor,
    ));
}

fn apply_bias(
    trigger: Trigger<OnAdd, BiasTowardsOrigin>,
    mut commands: Commands,
    anchor_query: Query<Entity, With<OriginAnchor>>,
) {
    let target_id = trigger.entity();
    let Ok(anchor_id) = anchor_query.get_single() else {
        warn!("Failed to get the origin anchor to create joint for {target_id}");
        return;
    };
    commands.spawn(
        DistanceJoint::new(anchor_id, target_id)
            .with_rest_length(5000.0)
            .with_linear_velocity_damping(0.05)
            .with_angular_velocity_damping(0.5)
            .with_limits(0.0, 8000.0)
            .with_compliance(0.000001),
    );
    debug!("Created a tether to the origin for {target_id}");
}
