#![feature(pattern, duration_constructors)]
mod accounts;
mod app;
pub mod cost_management;
mod duration;
mod eligible_child_resources;
mod fake;
mod groups;
mod management_group_ancestors_chain;
mod management_groups;
mod metrics;
mod naming;
mod oauth2_permission_grants;
mod oauth2_permission_scopes;
mod pim_azurerm_role_assignment_schedule_requests;
mod pim_entra_role_assignment_requests;
mod pim_entra_role_assignments;
mod pim_entra_role_definitions;
mod pim_entra_role_settings;
mod policy_assignments;
mod policy_definitions;
mod policy_set_definitions;
mod principals;
mod query_response;
mod resource_group_map;
mod resource_groups;
mod resource_tags;
mod resources;
mod role_assignments;
mod role_definitions;
mod role_eligibility_schedules;
mod role_management_policies;
mod role_management_policy_assignments;
mod scope_itertools;
mod scopes;
mod service_principal;
mod storage_accounts;
mod subscriptions;
mod tenants;
mod test_resource;
mod users;
mod uuid_macros;
mod uuid_wrapper;

pub mod prelude {
    pub use crate::accounts::*;
    pub use crate::app::*;
    pub use crate::duration::*;
    pub use crate::eligible_child_resources::*;
    pub use crate::fake::*;
    pub use crate::groups::*;
    pub use crate::management_group_ancestors_chain::*;
    pub use crate::management_groups::*;
    pub use crate::metrics::*;
    pub use crate::naming::*;
    pub use crate::oauth2_permission_grants::*;
    pub use crate::oauth2_permission_scopes::*;
    pub use crate::pim_azurerm_role_assignment_schedule_requests::*;
    pub use crate::pim_entra_role_assignment_requests::*;
    pub use crate::pim_entra_role_assignments::*;
    pub use crate::pim_entra_role_definitions::*;
    pub use crate::pim_entra_role_settings::*;
    pub use crate::policy_assignments::*;
    pub use crate::policy_definitions::*;
    pub use crate::policy_set_definitions::*;
    pub use crate::principals::*;
    pub use crate::query_response::*;
    pub use crate::resource_group_map::*;
    pub use crate::resource_groups::*;
    pub use crate::resource_tags::*;
    pub use crate::resources::*;
    pub use crate::role_assignments::*;
    pub use crate::role_definitions::*;
    pub use crate::role_eligibility_schedules::*;
    pub use crate::role_management_policies::*;
    pub use crate::role_management_policy_assignments::*;
    pub use crate::scope_itertools::*;
    pub use crate::scopes::*;
    pub use crate::service_principal::*;
    pub use crate::storage_accounts::*;
    pub use crate::subscriptions::*;
    pub use crate::tenants::*;
    pub use crate::test_resource::*;
    pub use crate::users::*;
    pub use crate::uuid_wrapper::*;
    pub use cloud_terrastodon_azure_resource_types::prelude::*;
    pub use uuid;
}
