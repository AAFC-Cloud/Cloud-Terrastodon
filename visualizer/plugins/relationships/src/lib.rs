#![feature(trivial_bounds)]
mod scope;
mod scope_tracking;

use bevy::prelude::*;
use prelude::ScopePlugin;
use prelude::ScopeTrackingPlugin;

pub struct RelationshipsPlugin;
impl Plugin for RelationshipsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ScopePlugin);
        app.add_plugins(ScopeTrackingPlugin);
    }
}

pub mod prelude {
    pub use crate::RelationshipsPlugin;
    pub use crate::scope::*;
    pub use crate::scope_tracking::*;
}