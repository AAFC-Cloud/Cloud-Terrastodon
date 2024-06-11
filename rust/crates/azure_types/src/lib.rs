mod groups;
mod management_groups;
mod policy_assignments;
mod policy_definitions;
mod policy_set_definitions;
mod resource_groups;
mod resource_name_rules;
mod role_assignments;
mod scopes;
mod subscriptions;
mod tenants;
mod users;
mod scope_itertools;
mod test_resource;

pub mod prelude {
    pub use crate::groups::*;
    pub use crate::management_groups::*;
    pub use crate::policy_assignments::*;
    pub use crate::policy_definitions::*;
    pub use crate::policy_set_definitions::*;
    pub use crate::resource_groups::*;
    pub use crate::resource_name_rules::*;
    pub use crate::role_assignments::*;
    pub use crate::scopes::*;
    pub use crate::subscriptions::*;
    pub use crate::tenants::*;
    pub use crate::users::*;
    pub use crate::scope_itertools::*;
    pub(crate) use crate::test_resource::*;
}
