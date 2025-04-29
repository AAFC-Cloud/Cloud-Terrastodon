#![feature(let_chains, async_fn_track_caller)]
mod azuredevops_git_repository_initialization_patcher;
mod block_lister;
mod body_formatter;
mod data_block_creation;
mod data_lookup_holder;
mod data_reference_patcher;
mod default_attribute_removal_patcher;
mod import_builder;
mod import_lookup_holder;
mod imported_resource_reference_patcher;
mod importer;
mod json_patcher;
mod provider_manager;
mod reflow;
mod sorting;
mod terraform_block_extracter_patcher;
mod user_id_reference_patcher;
mod work_dir_lifecycle;
mod writer;
pub mod prelude {
    pub use crate::block_lister::*;
    pub use crate::import_builder::*;
    pub use crate::importer::*;
    pub use crate::provider_manager::*;
    pub use crate::reflow::*;
    pub use crate::work_dir_lifecycle::*;
    pub use crate::writer::*;
    pub use cloud_terrastodon_core_tofu_types::prelude::*;
}
