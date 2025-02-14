#![feature(duration_constructors)]
mod azure_devops_projects;
mod azure_devops_queries;
mod azure_devops_repos;

pub mod prelude {
    pub use crate::azure_devops_projects::*;
    pub use crate::azure_devops_queries::*;
    pub use crate::azure_devops_repos::*;
    pub use cloud_terrastodon_core_azure_devops_types::prelude::*;
}
