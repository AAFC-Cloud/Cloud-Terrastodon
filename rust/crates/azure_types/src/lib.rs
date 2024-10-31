#![feature(pattern, duration_constructors)]
mod duration;
mod eligible_child_resources;
mod fake;
mod groups;
mod management_groups;
mod naming;
mod pim_azurerm_role_assignment_schedule_requests;
mod pim_entra_role_assignment_requests;
mod pim_entra_role_assignments;
mod pim_entra_role_definitions;
mod pim_entra_role_settings;
mod policy_assignments;
mod policy_definitions;
mod policy_set_definitions;
mod query_response;
mod resource_groups;
mod resources;
mod role_assignments;
mod role_definitions;
mod role_eligibility_schedules;
mod role_management_policies;
mod role_management_policy_assignments;
mod scope_itertools;
mod scopes;
mod security_groups;
mod storage_accounts;
mod subscriptions;
mod tenants;
mod test_resource;
mod users;
mod resource_tags;

pub mod prelude {
    pub use crate::resource_tags::*;
    pub use crate::duration::*;
    pub use crate::eligible_child_resources::*;
    pub use crate::fake::*;
    pub use crate::groups::*;
    pub use crate::management_groups::*;
    pub use crate::naming::*;
    pub use crate::pim_azurerm_role_assignment_schedule_requests::*;
    pub use crate::pim_entra_role_assignment_requests::*;
    pub use crate::pim_entra_role_assignments::*;
    pub use crate::pim_entra_role_definitions::*;
    pub use crate::pim_entra_role_settings::*;
    pub use crate::policy_assignments::*;
    pub use crate::policy_definitions::*;
    pub use crate::policy_set_definitions::*;
    pub use crate::query_response::*;
    pub use crate::resource_groups::*;
    pub use crate::resources::*;
    pub use crate::role_assignments::*;
    pub use crate::role_definitions::*;
    pub use crate::role_eligibility_schedules::*;
    pub use crate::role_management_policies::*;
    pub use crate::role_management_policy_assignments::*;
    pub use crate::scope_itertools::*;
    pub use crate::scopes::*;
    pub use crate::security_groups::*;
    pub use crate::storage_accounts::*;
    pub use crate::subscriptions::*;
    pub use crate::tenants::*;
    pub use crate::test_resource::*;
    pub use crate::users::*;
    pub use uuid;
}
