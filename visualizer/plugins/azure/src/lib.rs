#![feature(trivial_bounds, try_blocks)]
mod az_cli;
mod azure_devops_projects;
mod azure_devops_repos;
mod azure_users;
mod management_groups;
mod resource_groups;
mod role_assignments;
mod scope;
mod subscriptions;

use az_cli::AzureCliPlugin;
use azure_devops_projects::AzureDevOpsProjectsPlugin;
use bevy::prelude::*;
use prelude::AzureDevOpsReposPlugin;
use prelude::AzureUsersPlugin;
use prelude::ManagementGroupsPlugin;
use resource_groups::ResourceGroupsPlugin;
use role_assignments::RoleAssignmentsPlugin;
use scope::ScopePlugin;
use subscriptions::SubscriptionsPlugin;

pub struct AzurePlugin;
impl Plugin for AzurePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ResourceGroupsPlugin);
        app.add_plugins(SubscriptionsPlugin);
        app.add_plugins(AzureCliPlugin);
        app.add_plugins(ScopePlugin);
        app.add_plugins(AzureDevOpsProjectsPlugin);
        app.add_plugins(AzureDevOpsReposPlugin);
        app.add_plugins(AzureUsersPlugin);
        app.add_plugins(ManagementGroupsPlugin);
        app.add_plugins(RoleAssignmentsPlugin);
    }
}

pub mod prelude {
    pub use crate::*;
    pub use az_cli::*;
    pub use azure_devops_projects::*;
    pub use azure_devops_repos::*;
    pub use azure_users::*;
    pub use management_groups::*;
    pub use resource_groups::*;
    pub use role_assignments::*;
    pub use scope::*;
    pub use subscriptions::*;
}
