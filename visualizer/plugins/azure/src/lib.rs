#![feature(trivial_bounds)]
mod az_cli;
mod joints;
mod resource_groups;
mod scope;
mod subscriptions;
mod upright;

use az_cli::AzureCliPlugin;
use bevy::prelude::*;
use joints::JointsPlugin;
use resource_groups::ResourceGroupsPlugin;
use scope::ScopePlugin;
use subscriptions::SubscriptionsPlugin;
use upright::UprightPlugin;

pub struct AzurePlugin;
impl Plugin for AzurePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ResourceGroupsPlugin);
        app.add_plugins(SubscriptionsPlugin);
        app.add_plugins(AzureCliPlugin);
        app.add_plugins(ScopePlugin);
        app.add_plugins(JointsPlugin);
        app.add_plugins(UprightPlugin);
    }
}
