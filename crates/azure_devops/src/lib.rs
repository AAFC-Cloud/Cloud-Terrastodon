#![feature(duration_constructors_lite)]
mod azure_devops_group;
mod azure_devops_group_member;
mod azure_devops_projects;
mod azure_devops_repos;
mod azure_devops_service_endpoint;
mod azure_devops_team;
mod azure_devops_team_member;
mod azure_devops_user_license_entitlements;
mod azure_devops_work_item_queries;
mod azure_devops_work_item_queries_invoke;
mod get_default_organization_name;
mod get_default_project_name;
mod get_pat;
mod azure_devops_configure;

pub mod prelude {
    pub use crate::azure_devops_group::*;
    pub use crate::azure_devops_configure::*;
    pub use crate::azure_devops_group_member::*;
    pub use crate::azure_devops_projects::*;
    pub use crate::azure_devops_repos::*;
    pub use crate::azure_devops_service_endpoint::*;
    pub use crate::azure_devops_team::*;
    pub use crate::azure_devops_team_member::*;
    pub use crate::azure_devops_user_license_entitlements::*;
    pub use crate::azure_devops_work_item_queries::*;
    pub use crate::azure_devops_work_item_queries_invoke::*;
    pub use crate::get_default_organization_name::*;
    pub use crate::get_default_project_name::*;
    pub use crate::get_pat::*;
    pub use cloud_terrastodon_azure_devops_types::prelude::*;
}
