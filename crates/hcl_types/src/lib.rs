#![feature(let_chains)]
mod block;
mod code_reference;
mod data;
mod imports;
mod providers;
mod resources;
mod sanitize;
mod strings;
mod terraform_block;
mod terraform_registry_provider;
mod tf_work_dir;
mod users_lookup_body;
mod version;

pub mod prelude {
    pub use crate::block::*;
    pub use crate::code_reference::*;
    pub use crate::data::*;
    pub use crate::imports::*;
    pub use crate::providers::*;
    pub use crate::resources::*;
    pub use crate::sanitize::*;
    pub use crate::strings::*;
    pub use crate::terraform_block::*;
    pub use crate::terraform_registry_provider::*;
    pub use crate::tf_work_dir::*;
    pub use crate::users_lookup_body::*;
    pub use crate::version::*;
    pub use hcl::*;
}
