use crate::noninteractive::clean;
use eyre::Result;

/// Remove generated artifacts from previous runs.
#[derive(facet::Facet, Debug, Clone, Default)]
pub struct CleanArgs;

impl CleanArgs {
    pub async fn invoke(self) -> Result<()> {
        clean().await?;
        Ok(())
    }
}
