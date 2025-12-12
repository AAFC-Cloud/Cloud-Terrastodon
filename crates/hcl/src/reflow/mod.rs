//! Take an existing directory containing HCL and "reflow" it by changing what files the HCL structures live under.
//! Also perform various transformations to improve the quality of the HCL.
//!
//! # Rules
//!
//! ## Terraform blocks
//!
//! There can only be one `terraform` block, and it must live in `terraform.tf`.
//!
//! ## Provider blocks
//!
//! Each `provider` block without an alias must live in its own file named `provider.{provider_label}.tf`.
//!
//! Each `provider` block with an alias must live in its own file named `provider.{provider_label}.{alias}.tf`.
//!
//! ## Import blocks
//!
//! Any `import` block, if the `to` resource exists, must live located directly above the resource it imports.
//! If the `to` resource does not exist, the `import` block must live in its own file named `import.{resource_type}.{resource_name}.tf`.
//!
//! ## String attributes
//!
//! Any attribute whose value is a string literal that can successfully be parsed as JSON must be replaced with a call to `jsonencode(...)`.

mod reflow_json_attributes;
mod reflow_expressions_use_imported_resource_blocks;
mod reflow_remove_default_attributes;
mod reflow_trait;
mod reflow_new;
mod reflow_principal_id_comments;
mod reflow_azure_devops_git_repository_initialization_attributes;
mod reflow_by_block_identifier;

pub use reflow_json_attributes::*;
pub use reflow_expressions_use_imported_resource_blocks::*;
pub use reflow_remove_default_attributes::*;
pub use reflow_trait::*;
pub use reflow_new::*;
pub use reflow_principal_id_comments::*;
pub use reflow_azure_devops_git_repository_initialization_attributes::*;
pub use reflow_by_block_identifier::*;