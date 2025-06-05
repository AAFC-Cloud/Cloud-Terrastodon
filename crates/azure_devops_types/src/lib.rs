mod azure_devops_organization_name;
mod azure_devops_project;
mod azure_devops_project_id;
mod azure_devops_project_name;
mod azure_devops_repos;
mod azure_devops_team;
mod azure_devops_work_item_query;
mod azure_devops_work_items;
mod azure_devops_organization_url;
mod azure_devops_service_endpoint;
mod azure_devops_service_endpoint_id;
mod azure_devops_service_endpoint_name;

pub mod prelude {
    pub use crate::azure_devops_organization_name::*;
    pub use crate::azure_devops_service_endpoint_id::*;
    pub use crate::azure_devops_project::*;
    pub use crate::azure_devops_organization_url::*;
    pub use crate::azure_devops_service_endpoint::*;
    pub use crate::azure_devops_project_id::*;
    pub use crate::azure_devops_project_name::*;
    pub use crate::azure_devops_repos::*;
    pub use crate::azure_devops_team::*;
    pub use crate::azure_devops_work_item_query::*;
    pub use crate::azure_devops_work_items::*;
    pub use crate::azure_devops_service_endpoint_name::*;
}
