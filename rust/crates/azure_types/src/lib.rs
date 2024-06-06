pub mod groups;
pub mod management_groups;
pub mod policy_assignments;
pub mod policy_definitions;
pub mod policy_set_definitions;
pub mod resource_groups;
mod resource_name_rules;
pub mod role_assignments;
pub mod scopes;
mod subscriptions;
mod tenants;
pub mod users;

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
}
