//! Cloud Terrastodon meta crate.
//!
//! This crate provides:
//! - A binary entrypoint (main.rs) for the CLI.
//! - Optional feature-gated re-exports of the subcrates so downstream users can
//!   depend on a single crate and opt into only what they need, or enable the
//!   `full` feature to get everything.
//!
//! Features:
//! - `full`: Enables all features listed below.
//! - `pathing`, `config`, `azure`, `azure-types`, `azure-resource-types`,
//!   `azure-devops`, `azure-devops-types`, `hcl`, `hcl-types`, `user-input`,
//!   `command`, `zombies`, `relative-location`, `ui-ratatui`, `ui-egui`,
//!   `azure-locations`, `credentials`.

// One pub use per optional subcrate, behind its feature flag.
#[cfg(feature = "pathing")]
pub use cloud_terrastodon_pathing as pathing;

#[cfg(feature = "config")]
pub use cloud_terrastodon_config as config;

#[cfg(feature = "azure")]
pub use cloud_terrastodon_azure as azure;

#[cfg(feature = "azure-types")]
pub use cloud_terrastodon_azure_types as azure_types;

#[cfg(feature = "azure-resource-types")]
pub use cloud_terrastodon_azure_resource_types as azure_resource_types;

#[cfg(feature = "azure-devops")]
pub use cloud_terrastodon_azure_devops as azure_devops;

#[cfg(feature = "azure-devops-types")]
pub use cloud_terrastodon_azure_devops_types as azure_devops_types;

#[cfg(feature = "hcl")]
pub use cloud_terrastodon_hcl as hcl;

#[cfg(feature = "hcl-types")]
pub use cloud_terrastodon_hcl_types as hcl_types;

#[cfg(feature = "user-input")]
pub use cloud_terrastodon_user_input as user_input;

#[cfg(feature = "command")]
pub use cloud_terrastodon_command as command;

#[cfg(feature = "zombies")]
pub use cloud_terrastodon_zombies as zombies;

#[cfg(feature = "relative-location")]
pub use cloud_terrastodon_relative_location as relative_location;

#[cfg(feature = "ui-ratatui")]
pub use cloud_terrastodon_ui_ratatui as ui_ratatui;

#[cfg(feature = "ui-egui")]
pub use cloud_terrastodon_ui_egui as ui_egui;

// #[cfg(feature = "azure-locations")]
// pub use cloud_terrastodon_azure_locations as azure_locations;

#[cfg(feature = "credentials")]
pub use cloud_terrastodon_credentials as credentials;
