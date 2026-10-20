//! Consumer-owned command schema and dispatch for the group reconciliation example.

mod command;
mod group_management_cli;

pub use group_management_cli::GroupManagementArgs;
pub use group_management_cli::GroupManagementCommand;
