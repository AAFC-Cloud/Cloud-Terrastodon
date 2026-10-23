mod azure_devops_work_item_cli;
pub mod copy;
pub mod create;
pub mod field;
pub mod list;
pub mod query;
pub mod relation;
pub mod show;
pub mod r#type;
pub mod update;
pub use azure_devops_work_item_cli::*;

#[cfg(test)]
mod azure_devops_work_item_cli_tests;
