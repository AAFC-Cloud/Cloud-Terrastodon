#![feature(let_chains, try_blocks)]
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
mod reflow;
mod sorting;
mod user_id_reference_patcher;
mod writer;
pub mod prelude {
    pub use crate::block_lister::*;
    pub use crate::import_builder::*;
    pub use crate::importer::*;
    pub use crate::reflow::*;
    pub use crate::writer::*;
    pub use cloud_terrastodon_core_tofu_types::prelude::*;
}
