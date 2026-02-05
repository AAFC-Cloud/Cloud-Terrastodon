pub mod clean_tui;
pub mod list;

use crate::cli::cache::clean_tui::CacheCleanTuiArgs;
use crate::cli::cache::list::cache_list_cli::CacheListArgs;
use clap::Args;
use clap::Subcommand;
use eyre::Result;

/// Cache-related commands
#[derive(Args, Debug, Clone)]
pub struct CacheArgs {
    #[command(subcommand)]
    pub command: CacheCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum CacheCommand {
    /// List cache entries
    List(CacheListArgs),
    /// Interactively select cache entries to invalidate (TUI)
    CleanTui(CacheCleanTuiArgs),
}

impl CacheArgs {
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}

impl CacheCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            CacheCommand::List(args) => args.invoke().await,
            CacheCommand::CleanTui(args) => args.invoke().await,
        }
    }
}
