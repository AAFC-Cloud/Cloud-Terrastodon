mod node_spawning;

use bevy::prelude::*;
use prelude::NodeSpawningPlugin;

pub struct GraphNodesPlugin;
impl Plugin for GraphNodesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(NodeSpawningPlugin);
    }
}

pub mod prelude {
    pub use crate::*;
    pub use crate::node_spawning::*;
}
