//! This crate provides utilities for building, running, and managing external commands within the Cloud Terrastodon project. It includes features for:
//! 
//! - Specifying different command kinds (Azure CLI, Terraform, VSCode, Echo, Pwsh).
//! - Building command arguments and environment variables.
//! - Handling file arguments for commands like Azure CLI.
//! - Configuring output behavior (capture or display).
//! - Implementing retry logic for authentication failures.
//! - Caching command output for improved performance.
//! - Sending content to command stdin.
//! - Writing command failures and successes to files for debugging and caching.

#![feature(
    duration_constructors,
    string_from_utf8_lossy_owned,
    async_fn_track_caller
)]
pub mod app_work;
mod cachable_command;
mod cache_invalidatable;
mod cache_key;
mod command;
mod command_argument;
mod command_kind;
mod command_output;
mod no_spaces;
mod path_mapper;
mod work;

pub use crate::cachable_command::*;
pub use crate::cache_invalidatable::*;
pub use crate::cache_key::*;
pub use crate::command::*;
pub use crate::command_argument::*;
pub use crate::command_kind::*;
pub use crate::command_output::*;
pub use crate::no_spaces::*;
pub use crate::path_mapper::*;
pub use crate::work::*;
// Re-export async_trait for use in command implementations
pub use async_trait::async_trait;

// TODO: add a `last_used` file to cache entries so we can
// give the user the list of recently used cache entries to
// let the user surgically bust individual caches instead of only being able to clear the entire cache
