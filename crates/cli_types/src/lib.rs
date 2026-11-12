//! Shared command-line input contracts used by multiple command groups.

mod cloud_terrastodon_account_argument;
mod http_method_cli;
mod humantime_duration_cli;

pub use cloud_terrastodon_account_argument::CloudTerrastodonAccountArgument;
pub use http_method_cli::HttpMethodCli;
pub use humantime_duration_cli::HumantimeDurationCli;
