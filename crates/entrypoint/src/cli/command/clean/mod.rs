use crate::noninteractive::prelude::clean;
use clap::Args;
use eyre::Result;

/// Remove generated artifacts from previous runs.
#[derive(Args, Debug, Clone, Default)]
pub struct CleanArgs;

impl CleanArgs {
    pub async fn invoke(self) -> Result<()> {
        clean().await?;
        Ok(())
    }
}
