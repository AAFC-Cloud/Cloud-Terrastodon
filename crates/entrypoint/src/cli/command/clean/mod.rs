use crate::noninteractive::clean;
use cloud_terrastodon_cache::WorkCache;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use facet::Facet;

/// Remove generated artifacts from previous runs.
#[derive(Facet, Debug, Clone, Default)]
pub struct CleanArgs;

impl CleanArgs {
    pub async fn invoke(self, cache: &WorkCache, cancellation: &CancellationScope) -> Result<()> {
        clean(cache, cancellation).await?;
        Ok(())
    }
}
