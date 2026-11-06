use cloud_terrastodon_command::CancellationScope;
use facet::Facet;
use tracing::info;

/// Emit a message through the configured logging pipeline.
#[derive(Facet, Debug, Clone)]
pub struct EchoArgs {
    /// The message to emit.
    #[facet(figue::positional, default)]
    pub message: Vec<String>,
}

impl EchoArgs {
    pub async fn invoke(self, _cancellation: &CancellationScope) -> eyre::Result<()> {
        info!("{}", self.message.join(" "));
        Ok(())
    }
}
