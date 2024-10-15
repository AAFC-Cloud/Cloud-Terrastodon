mod bias_towards_origin;
mod joints;
mod organize;
mod upright;
mod alignment_plugin;

use bevy::prelude::*;
use bias_towards_origin::BiasPlugin;
use alignment_plugin::AlignmentPlugin;
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
    pub use bias_towards_origin::*;
    pub use joints::*;
    pub use organize::*;
    pub use upright::*;
    pub use alignment_plugin::*;
}
