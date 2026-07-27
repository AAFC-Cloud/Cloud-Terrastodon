use tracing::info;

/// Emit a message through the configured logging pipeline.
#[derive(facet::Facet, Debug, Clone)]
pub struct EchoArgs {
    /// The message to emit.
    #[facet(figue::positional, default)]
    pub message: Vec<String>,
}

impl EchoArgs {
    pub async fn invoke(self) -> eyre::Result<()> {
        info!("{}", self.message.join(" "));
        Ok(())
    }
}
