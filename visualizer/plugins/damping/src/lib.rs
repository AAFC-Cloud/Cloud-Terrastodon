use avian2d::prelude::LinearVelocity;
use avian2d::prelude::Sleeping;
use bevy::prelude::*;

pub struct DampingPlugin;

impl Plugin for DampingPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CustomLinearDamping>();
        app.add_systems(Update, apply);
    }
}

#[derive(Component, Debug, Reflect)]
pub struct CustomLinearDamping {
    pub retention_factor: f32,
    pub rest_threshold: f32,
}
impl Default for CustomLinearDamping {
    fn default() -> Self {
        Self {
            retention_factor: 0.93,
            rest_threshold: 10.0,
        }
    }
}

fn apply(mut damping_query: Query<(&mut LinearVelocity, &CustomLinearDamping), Without<Sleeping>>) {
    for thing in damping_query.iter_mut() {
        let (mut thing_velocity, thing_damping) = thing;
        thing_velocity.0 *= thing_damping.retention_factor;
        if thing_velocity
            .0
            .abs()
            .cmple(Vec2::splat(thing_damping.rest_threshold))
            .all()
        {
            thing_velocity.0 = Vec2::ZERO;
        }
    }
}
