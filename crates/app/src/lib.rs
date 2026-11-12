//! Shared application startup without a prescribed command tree.
//!
//! Define a public root CLI, flatten [`GlobalArgs`], implement [`ApplicationCli`],
//! then pass your handler to [`App::run`]. App supplies Figue's builtin flags
//! through an internal parsing schema. Neither `Debug`, `Arbitrary`, nor registry
//! registration is required on your CLI.
//!
//! `run` owns the process-global error, tracing and signal hooks and a Tokio
//! runtime: call it once, from synchronous `main`. Libraries and tests that only
//! need argument parsing should use [`App::parse_from`] instead. That method does
//! not install startup hooks or resolve authentication; Figue's explicitly
//! requested help/schema export facilities retain their usual behavior.
//! Creating an `eyre::Report` before startup may itself install the default error
//! hook; finish shared startup before creating application error reports.

mod app;
mod app_cli;
mod app_context;
mod application_cli;
mod cancel_on_drop;
mod global_args;
pub mod output;
mod startup;
#[cfg(feature = "terminal")]
mod terminal;
#[cfg(windows)]
mod windows_support;

pub use app::App;
pub use app_context::AppContext;
pub use application_cli::ApplicationCli;
pub use eyre::Result;
pub use facet;
pub use figue;
pub use global_args::GlobalArgs;
pub use output::CliOutput;
pub use output::CliOutputValue;
pub use output::OutputFormat;
pub use output::write_facet;
pub use startup::install_error_hook;
pub use startup::prepare_process;
pub use teamy_cancellation::CancellationReportExt;
pub use teamy_cancellation::CancellationScope;
pub use tracing;

#[cfg(test)]
mod tests;
