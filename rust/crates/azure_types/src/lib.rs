#![feature(pattern)]
mod duration;
mod eligible_child_resources;
mod groups;
mod management_groups;
mod policy_assignments;
mod policy_definitions;
mod policy_set_definitions;
mod query_response;
mod resource_groups;
mod resource_name_rules;
mod role_assignment_schedule_requests;
mod role_assignments;
mod role_definitions;
mod role_eligibility_schedules;
mod role_management_policies;
mod role_management_policy_assignments;
mod scope_itertools;
mod scopes;
mod subscriptions;
mod tenants;
mod test_resource;
mod users;

pub mod prelude {
    pub use crate::duration::*;
    pub use crate::eligible_child_resources::*;
    pub use crate::groups::*;
    pub use crate::management_groups::*;
    pub use crate::policy_assignments::*;
    pub use crate::policy_definitions::*;
    pub use crate::policy_set_definitions::*;
    pub use crate::query_response::*;
    pub use crate::resource_groups::*;
    pub use crate::resource_name_rules::*;
    pub use crate::role_assignment_schedule_requests::*;
    pub use crate::role_assignments::*;
    pub use crate::role_definitions::*;
    pub use crate::role_eligibility_schedules::*;
    pub use crate::role_management_policies::*;
    pub use crate::role_management_policy_assignments::*;
    pub use crate::scope_itertools::*;
    pub use crate::scopes::*;
    pub use crate::subscriptions::*;
    pub use crate::tenants::*;
    pub(crate) use crate::test_resource::*;
    pub use crate::users::*;
    pub use uuid;
}
