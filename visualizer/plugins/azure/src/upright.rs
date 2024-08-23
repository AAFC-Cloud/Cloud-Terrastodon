use crate::scope::AzureScope;
use avian2d::prelude::AngularVelocity;
use avian2d::prelude::Rotation;
use bevy::prelude::*;

pub struct UprightPlugin;
impl Plugin for UprightPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, fix_rotation);
    }
}

// we want scopey things to always remain upright
fn fix_rotation(mut query: Query<(&mut AngularVelocity, &mut Rotation), With<AzureScope>>) {
    for thing in query.iter_mut() {
        let (mut angular_velocity, mut rotation) = thing;
        angular_velocity.0 = 0.0;
        rotation.sin = 0.;
        rotation.cos = 1.;
    }
}
