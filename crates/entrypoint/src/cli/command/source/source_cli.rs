use super::SourceHuntArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;

/// Inspect source code stored in local repositories.
#[derive(Debug, Clone, facet::Facet)]
pub struct SourceArgs {
    #[facet(figue::subcommand)]
    pub command: SourceCommand,
}

impl SourceArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        self.command.invoke(cancellation).await
    }
}

/// Local source discovery operations.
#[derive(Debug, Clone, facet::Facet)]
#[repr(u8)]
pub enum SourceCommand {
    /// Find Git repositories recursively and summarize file extensions and authors.
    Hunt(SourceHuntArgs),
}

impl SourceCommand {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        match self {
            Self::Hunt(args) => Box::pin(args.invoke(cancellation)).await,
        }
    }
}

cloud_terrastodon_registry::register_thing!(SourceArgs);
cloud_terrastodon_registry::register_thing!(SourceCommand);
