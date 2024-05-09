pub mod management_groups;
pub mod policy_assignments;
pub mod policy_definitions;
pub mod policy_set_definitions;
pub mod scopes;
mod groups;
mod users;

pub mod prelude {
    pub use crate::management_groups::*;
    pub use crate::policy_assignments::*;
    pub use crate::policy_definitions::*;
    pub use crate::policy_set_definitions::*;
    pub use crate::groups::*;
    pub use crate::scopes::*;
    pub use crate::users::*;
}
