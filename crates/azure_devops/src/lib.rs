#![feature(duration_constructors_lite)]
mod azure_devops_projects;
mod azure_devops_repos;
mod azure_devops_teams;
mod azure_devops_work_item_queries;
mod azure_devops_work_item_queries_invoke;
mod get_default_organization_name;
mod get_default_project_name;
mod get_pat;

pub mod prelude {
    pub use crate::azure_devops_projects::*;
    pub use crate::azure_devops_repos::*;
    pub use crate::azure_devops_teams::*;
    pub use crate::azure_devops_work_item_queries::*;
    pub use crate::azure_devops_work_item_queries_invoke::*;
    pub use crate::get_default_organization_name::*;
    pub use crate::get_default_project_name::*;
    pub use crate::get_pat::*;
    pub use cloud_terrastodon_azure_devops_types::prelude::*;
}
