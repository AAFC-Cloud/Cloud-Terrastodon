#![feature(trivial_bounds)]
mod az_cli;
mod resource_groups;
mod scope;
mod subscriptions;

use az_cli::AzureCliPlugin;
use bevy::prelude::*;
use resource_groups::ResourceGroupsPlugin;
use scope::ScopePlugin;
use subscriptions::SubscriptionsPlugin;

pub struct AzurePlugin;
impl Plugin for AzurePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ResourceGroupsPlugin);
        app.add_plugins(SubscriptionsPlugin);
        app.add_plugins(AzureCliPlugin);
        app.add_plugins(ScopePlugin);
    }
}

pub mod prelude {
    pub use crate::*;
    pub use scope::*;
    pub use resource_groups::*;
    pub use subscriptions::*;
    pub use az_cli::*;
}