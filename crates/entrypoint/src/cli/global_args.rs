use clap::Args;
use std::path::PathBuf;

/// Arguments that apply to all commands.
#[derive(Args, Debug, Clone)]
pub struct GlobalArgs {
    /// Enable debug logging, including backtraces on panics.
    #[arg(long, global = true, default_value_t = false)]
    pub debug: bool,

    /// Write structured ndjson logs to this file or directory. If a directory is provided,
    /// a filename will be generated there. If omitted, no JSON log file will be written.
    #[arg(long, global = true, value_name = "FILE|DIR")]
    pub log_file: Option<PathBuf>,
}
