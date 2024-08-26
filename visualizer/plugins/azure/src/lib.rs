#![feature(trivial_bounds)]
mod az_cli;
mod joints;
mod resource_groups;
mod scope;
mod subscriptions;
mod upright;
mod bais_towards_origin;

use az_cli::AzureCliPlugin;
use bevy::prelude::*;
use joints::JointsPlugin;
use resource_groups::ResourceGroupsPlugin;
use scope::ScopePlugin;
use subscriptions::SubscriptionsPlugin;
use upright::UprightPlugin;
use bais_towards_origin::BiasPlugin;

pub struct AzurePlugin;
impl Plugin for AzurePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ResourceGroupsPlugin);
        app.add_plugins(SubscriptionsPlugin);
        app.add_plugins(AzureCliPlugin);
        app.add_plugins(ScopePlugin);
        app.add_plugins(JointsPlugin);
        app.add_plugins(UprightPlugin);
        app.add_plugins(BiasPlugin);
    }
}
