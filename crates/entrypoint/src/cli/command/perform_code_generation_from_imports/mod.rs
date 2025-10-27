use crate::noninteractive::prelude::perform_import;
use crate::noninteractive::prelude::process_generated;
use clap::Args;
use eyre::Result;

/// Perform code-generation from existing import definitions.
#[derive(Args, Debug, Clone, Default)]
pub struct PerformCodeGenerationFromImportsArgs;

impl PerformCodeGenerationFromImportsArgs {
    pub async fn invoke(self) -> Result<()> {
        perform_import().await?;
        process_generated().await?;
        Ok(())
    }
}
