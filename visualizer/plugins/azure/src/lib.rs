#![feature(trivial_bounds, try_blocks)]
mod az_cli;
mod azure_devops_projects;
mod azure_devops_repos;
mod resource_groups;
mod scope;
mod subscriptions;
mod azure_users;
mod management_groups;

use az_cli::AzureCliPlugin;
use azure_devops_projects::AzureDevopsProjectsPlugin;
use bevy::prelude::*;
use prelude::AzureDevopsReposPlugin;
use prelude::AzureUsersPlugin;
use prelude::ManagementGroupsPlugin;
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
        app.add_plugins(AzureUsersPlugin);
        app.add_plugins(ManagementGroupsPlugin);
    }
}

pub mod prelude {
    pub use crate::*;
    pub use az_cli::*;
    pub use azure_devops_projects::*;
    pub use azure_devops_repos::*;
    pub use resource_groups::*;
    pub use scope::*;
    pub use subscriptions::*;
    pub use azure_users::*;
    pub use management_groups::*;
}
