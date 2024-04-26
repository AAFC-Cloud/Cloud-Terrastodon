pub mod management_groups;
pub mod policy_assignments;
pub mod policy_definitions;
pub mod policy_set_definitions;
pub mod scopes;

pub mod prelude {
    pub use crate::management_groups::*;
    pub use crate::policy_assignments::*;
    pub use crate::policy_definitions::*;
    pub use crate::policy_set_definitions::*;
    pub use crate::scopes::*;
}
