#![feature(async_fn_track_caller)]
mod audit;
mod block_lister;
mod body_formatter;
mod data_lookup_holder;
mod data_reference_patcher;
mod discover_recursive_source_dirs;
pub mod discovery;
mod import_builder;
mod importer;
mod provider_manager;
pub mod reflow;
mod sorting;
mod terraform_block_extracter_patcher;
mod work_dir_lifecycle;
mod writer;
pub mod prelude {
    pub use crate::audit::*;
    pub use crate::block_lister::*;
    pub use crate::data_reference_patcher::*;
    pub use crate::discover_recursive_source_dirs::*;
    pub use crate::import_builder::*;
    pub use crate::importer::*;
    pub use crate::provider_manager::*;
    pub use crate::terraform_block_extracter_patcher::*;
    pub use crate::work_dir_lifecycle::*;
    pub use crate::writer::*;
    pub use cloud_terrastodon_hcl_types::prelude::*;
}
