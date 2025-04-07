#![feature(impl_trait_in_assoc_type, duration_constructors)]
mod auth;
mod batch_api;
mod cost_management;
mod create_role_assignment;
mod eligible_child_resources;
mod evaluate_policy_assignment_compliance;
mod groups;
mod management_groups;
mod metrics;
mod microsoft_graph;
mod name_lookup_helper;
mod oauth2_permission_grants;
mod oauth2_permission_scopes;
mod pick_oauth2_permission_grants;
mod pim_azurerm_activate;
mod pim_entra_activate;
mod pim_entra_role_assignments;
mod pim_entra_role_definitions;
mod pim_entra_role_settings;
mod policy_assignments;
mod policy_definitions;
mod policy_set_definitions;
mod principals;
mod remediate_policy_assignment;
pub mod remove_oauth2_permission_grant;
mod resource_graph;
mod resource_groups;
mod resource_types_generator;
mod resources;
mod role_assignments;
mod role_definitions;
mod role_eligibility_schedules;
mod role_management_policy_assignments;
mod security_groups;
mod service_principals;
mod storage_accounts;
mod subscriptions;
mod tags;
mod users;
mod tenants;
pub mod prelude {
    pub use crate::auth::*;
    pub use crate::tenants::*;
    pub use crate::batch_api::*;
    pub use crate::cost_management::*;
    pub use crate::create_role_assignment::*;
    pub use crate::eligible_child_resources::*;
    pub use crate::evaluate_policy_assignment_compliance::*;
    pub use crate::groups::*;
    pub use crate::management_groups::*;
    pub use crate::metrics::*;
    pub use crate::microsoft_graph::*;
    pub use crate::name_lookup_helper::*;
    pub use crate::oauth2_permission_grants::*;
    pub use crate::oauth2_permission_scopes::*;
    pub use crate::pick_oauth2_permission_grants::*;
    pub use crate::pim_azurerm_activate::*;
    pub use crate::pim_entra_activate::*;
    pub use crate::pim_entra_role_assignments::*;
    pub use crate::pim_entra_role_definitions::*;
    pub use crate::pim_entra_role_settings::*;
    pub use crate::policy_assignments::*;
    pub use crate::policy_definitions::*;
    pub use crate::policy_set_definitions::*;
    pub use crate::principals::*;
    pub use crate::remediate_policy_assignment::*;
    pub use crate::remove_oauth2_permission_grant::*;
    pub use crate::resource_graph::*;
    pub use crate::resource_groups::*;
    pub use crate::resources::*;
    pub use crate::role_assignments::*;
    pub use crate::role_definitions::*;
    pub use crate::role_eligibility_schedules::*;
    pub use crate::role_management_policy_assignments::*;
    pub use crate::security_groups::*;
    pub use crate::service_principals::*;
    pub use crate::storage_accounts::*;
    pub use crate::subscriptions::*;
    pub use crate::tags::*;
    pub use crate::users::*;
    pub use cloud_terrastodon_core_azure_types::prelude::*;
}
