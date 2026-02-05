use clap::Args;
use cloud_terrastodon_azure::prelude::fetch_all_groups;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// List Entra (Azure AD) groups.
#[derive(Args, Debug, Clone)]
pub struct AzureEntraGroupListArgs {}

impl AzureEntraGroupListArgs {
    pub async fn invoke(self) -> Result<()> {
        info!("Fetching Entra groups");
        let groups = fetch_all_groups().await?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &groups)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
