use crate::noninteractive::prelude::dump_everything;
use clap::Args;
use eyre::Result;

/// Dump all collected metadata to disk.
#[derive(Args, Debug, Clone, Default)]
pub struct DumpEverythingArgs;

impl DumpEverythingArgs {
    pub async fn invoke(self) -> Result<()> {
        dump_everything().await?;
        Ok(())
    }
}
