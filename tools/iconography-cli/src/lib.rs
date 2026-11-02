//! Offline catalogue generation and public-source updates for the standalone
//! `iconography` maintenance application.
//!
//! These operations belong to this unpublished tool. Icon consumers depend on
//! the three asset/library crates and do not depend on this package.

pub mod cli;
mod generated_file;
pub mod generation;
mod github_branch_reference;
mod github_object_reference;
mod icon_catalogue;
mod icon_catalogue_entry;
mod icon_family;
pub mod importer;
mod operation_paths_proxy;
mod operation_report;
mod public_asset_download;
mod source_discovery;
pub mod validation;

pub use github_branch_reference::GitHubBranchReference;
pub use github_object_reference::GitHubObjectReference;
pub use icon_catalogue::IconCatalogue;
pub use icon_catalogue_entry::IconCatalogueEntry;
pub use icon_family::IconFamily;
pub use operation_report::OperationReport;
