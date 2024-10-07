#![feature(trivial_bounds, try_blocks)]
mod az_cli;
mod resource_groups;
mod scope;
mod subscriptions;
mod azure_devops_projects;
mod azure_devops_repos;

use az_cli::AzureCliPlugin;
use azure_devops_projects::AzureDevopsProjectsPlugin;
use bevy::prelude::*;
use prelude::AzureDevopsReposPlugin;
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
        app.add_plugins(AzureDevopsProjectsPlugin);
        app.add_plugins(AzureDevopsReposPlugin);
    }
}

pub mod prelude {
    pub use crate::*;
    pub use scope::*;
    pub use resource_groups::*;
    pub use azure_devops_projects::*;
    pub use azure_devops_repos::*;
    pub use subscriptions::*;
    pub use az_cli::*;
}