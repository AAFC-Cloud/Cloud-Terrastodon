use crate::noninteractive::clean;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;

/// Remove generated artifacts from previous runs.
#[derive(facet::Facet, Debug, Clone, Default)]
pub struct CleanArgs;

impl CleanArgs {
    pub async fn invoke(self, _cancellation: &CancellationScope) -> Result<()> {
        clean().await?;
        Ok(())
    }
}
