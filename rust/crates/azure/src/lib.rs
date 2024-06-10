#![feature(async_closure, impl_trait_in_assoc_type)]
mod auth;
mod groups;
mod management_groups;
mod policy_assignments;
mod policy_definitions;
mod policy_set_definitions;
mod resource_groups;
mod subscriptions;
mod users;
mod name_lookup_helper;
mod role_assignments;
pub mod prelude {
    pub use crate::auth::*;
    pub use crate::groups::*;
    pub use crate::management_groups::*;
    pub use crate::policy_assignments::*;
    pub use crate::policy_definitions::*;
    pub use crate::policy_set_definitions::*;
    pub use crate::role_assignments::*;
    pub use crate::resource_groups::*;
    pub use crate::subscriptions::*;
    pub use crate::users::*;
    pub use crate::name_lookup_helper::*;
    pub use azure_types::prelude::*;
}
