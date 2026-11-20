//! Shared command-line input contracts used by multiple command groups.

mod http_method_cli;
mod humantime_duration_cli;

pub use cloud_terrastodon_auth_types::CloudTerrastodonAccountArgument;
pub use http_method_cli::HttpMethodCli;
pub use humantime_duration_cli::HumantimeDurationCli;
