#![feature(async_closure, impl_trait_in_assoc_type, duration_constructors)]
mod auth;
mod evaluate_policy_assignment_compliance;
mod groups;
mod management_groups;
mod name_lookup_helper;
mod policy_assignments;
mod policy_definitions;
mod policy_set_definitions;
mod remediate_policy_assignment;
mod resource_groups;
mod role_assignments;
mod subscriptions;
mod users;
mod query;
mod role_definitions;
mod role_eligibility_schedules;
mod eligible_child_resources;
pub mod prelude {
    pub use crate::auth::*;
    pub use crate::evaluate_policy_assignment_compliance::*;
    pub use crate::groups::*;
    pub use crate::management_groups::*;
    pub use crate::name_lookup_helper::*;
    pub use crate::policy_assignments::*;
    pub use crate::policy_definitions::*;
    pub use crate::policy_set_definitions::*;
    pub use crate::remediate_policy_assignment::*;
    pub use crate::resource_groups::*;
    pub use crate::role_assignments::*;
    pub use crate::role_definitions::*;
    pub use crate::subscriptions::*;
    pub use crate::role_eligibility_schedules::*;
    pub use crate::users::*;
    pub use crate::query::*;
    pub use crate::eligible_child_resources::*;
    pub use azure_types::prelude::*;
}
