//! Shared command-line input contracts used by multiple command groups.

mod http_method_cli;
mod humantime_duration_cli;

pub use http_method_cli::HttpMethodCli;
pub use humantime_duration_cli::HumantimeDurationCli;
