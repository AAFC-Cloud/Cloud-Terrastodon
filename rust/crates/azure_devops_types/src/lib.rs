mod azure_devops_projects;
mod azure_devops_repos;

pub mod prelude {
    pub use crate::azure_devops_projects::*;
    pub use crate::azure_devops_repos::*;
}