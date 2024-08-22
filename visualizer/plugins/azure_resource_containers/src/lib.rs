#![feature(trivial_bounds)]
mod az_cli;
mod resource_groups;
mod subscriptions;

use az_cli::AzureCliPlugin;
use bevy::prelude::*;
use resource_groups::ResourceGroupsPlugin;
use subscriptions::SubscriptionsPlugin;

pub struct AzureResourceContainersPlugin;
impl Plugin for AzureResourceContainersPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ResourceGroupsPlugin);
        app.add_plugins(SubscriptionsPlugin);
        app.add_plugins(AzureCliPlugin);
    }
}
