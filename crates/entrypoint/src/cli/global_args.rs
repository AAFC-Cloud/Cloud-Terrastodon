use clap::Args;

/// Arguments that apply to all commands.
#[derive(Args, Debug, Clone, Copy, Default)]
pub struct GlobalArgs {
    /// Enable debug logging, including backtraces on panics.
    #[arg(long, global = true, default_value_t = false)]
    pub debug: bool,

    /// Write a json file of the structured logs to the current directory.
    #[arg(long, global = true, default_value_t = false)]
    pub json: bool,
}
