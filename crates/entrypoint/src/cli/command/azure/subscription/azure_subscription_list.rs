use clap::Args;
use cloud_terrastodon_azure::prelude::fetch_all_subscriptions;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Arguments for listing Azure subscriptions.
#[derive(Args, Debug, Clone)]
pub struct AzureSubscriptionListArgs {}

impl AzureSubscriptionListArgs {
    pub async fn invoke(self) -> Result<()> {
        info!("Fetching all Azure subscriptions");
        let subs = fetch_all_subscriptions().await?;
        info!(count = subs.len(), "Fetched subscriptions");

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &subs)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}