#![feature(duration_constructors)]
mod azure_devops_projects;
mod azure_devops_queries;
mod azure_devops_repos;
mod get_default_organization_name;
mod get_default_project_name;
mod flatten_queries;

pub mod prelude {
    pub use crate::azure_devops_projects::*;
    pub use crate::azure_devops_queries::*;
    pub use crate::azure_devops_repos::*;
    pub use crate::get_default_organization_name::*;
    pub use crate::get_default_project_name::*;
    pub use crate::flatten_queries::*;
    pub use cloud_terrastodon_core_azure_devops_types::prelude::*;
}
