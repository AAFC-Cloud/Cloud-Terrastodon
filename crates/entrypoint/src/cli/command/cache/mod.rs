use facet::Facet;
pub mod clean_tui_cli;
pub mod list;

use crate::cli::cache::clean_tui_cli::CacheCleanTuiArgs;
use crate::cli::cache::list::cache_list_cli::CacheListArgs;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;

/// Cache-related commands
#[derive(Facet, Debug, Clone)]
pub struct CacheArgs {
    #[facet(figue::subcommand)]
    pub command: CacheCommand,
}

#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum CacheCommand {
    /// List cache entries
    List(CacheListArgs),
    /// Interactively select cache entries to invalidate (TUI)
    CleanTui(CacheCleanTuiArgs),
}

impl CacheArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<()> {
        self.command.invoke(executor).await
    }
}

impl CacheCommand {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<()> {
        match self {
            CacheCommand::List(args) => args.invoke(executor).await,
            CacheCommand::CleanTui(args) => args.invoke(executor).await,
        }
    }
}
