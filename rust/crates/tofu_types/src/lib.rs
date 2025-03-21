#![feature(let_chains)]
mod block;
mod code_reference;
mod data;
mod imports;
mod providers;
mod resources;
mod strings;
mod terraform_block;
mod version;
mod tf_work_dir;

pub mod prelude {
    pub use crate::block::*;
    pub use crate::version::*;
    pub use crate::code_reference::*;
    pub use crate::data::*;
    pub use crate::imports::*;
    pub use crate::providers::*;
    pub use crate::resources::*;
    pub use crate::strings::*;
    pub use crate::terraform_block::*;
    pub use crate::tf_work_dir::*;
    pub use hcl::*;
}
