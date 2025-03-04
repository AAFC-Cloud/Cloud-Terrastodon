mod alignment_plugin;
mod bias_towards_origin;
mod joints;
mod organize;
mod upright;

use alignment_plugin::AlignmentPlugin;
use bevy::prelude::*;
use bias_towards_origin::BiasPlugin;
use joints::JointsPlugin;
use organize::OrganizerPlugin;
use upright::UprightPlugin;

pub struct LayoutPlugin;

impl Plugin for LayoutPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(OrganizerPlugin);
        app.add_plugins(UprightPlugin);
        app.add_plugins(BiasPlugin);
        app.add_plugins(JointsPlugin);
        app.add_plugins(AlignmentPlugin);
    }
}

pub mod prelude {
    pub use crate::*;
    pub use alignment_plugin::*;
    pub use bias_towards_origin::*;
    pub use joints::*;
    pub use organize::*;
    pub use upright::*;
}
