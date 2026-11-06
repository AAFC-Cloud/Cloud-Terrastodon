//! This crate provides utilities for building, running, and managing external commands within the Cloud Terrastodon project. It includes features for:
//!
//! - Specifying different command kinds (Azure CLI, Terraform, VSCode, Echo, Pwsh).
//! - Building command arguments and environment variables.
//! - Handling file arguments for commands like Azure CLI.
//! - Configuring output behavior (capture or display).
//! - Retrying rate-limited subprocess attempts within their execution owner.
//! - Sharing canonical decoded command results through an explicit executor.
//! - Sending content to command stdin.
//! - Writing command failures and successes to files for debugging and caching.

pub mod app_work;
mod command;
mod command_argument;
mod command_input_workspace;
mod command_kind;
mod command_output;
mod command_process;
mod from_command_output;
mod output_behaviour;
mod path_mapper;
mod process_fingerprint;
mod retry_behaviour;
mod work;

pub use crate::command::*;
pub use crate::command_argument::*;
pub use crate::command_input_workspace::CommandInputWorkspace;
pub use crate::command_kind::*;
pub use crate::command_output::*;
pub use crate::from_command_output::FromCommandOutput;
pub use crate::output_behaviour::OutputBehaviour;
pub use crate::path_mapper::*;
pub use crate::retry_behaviour::RetryBehaviour;
pub use crate::work::*;
// Re-export async_trait for use in command implementations
pub use async_trait::async_trait;
pub use teamy_cancellation::CancellationError;
pub use teamy_cancellation::CancellationReportExt;
pub use teamy_cancellation::CancellationScope;
pub use teamy_cancellation::CancellationToken;

// TODO: add a `last_used` file to cache entries so we can
// give the user the list of recently used cache entries to
// let the user surgically bust individual caches instead of only being able to clear the entire cache
