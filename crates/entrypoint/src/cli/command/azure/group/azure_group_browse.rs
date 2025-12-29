use clap::Args;
use cloud_terrastodon_azure::prelude::fetch_all_resource_groups;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Arguments for browsing Azure resource groups.
#[derive(Args, Debug, Clone)]
pub struct AzureGroupBrowseArgs {}

impl AzureGroupBrowseArgs {
    pub async fn invoke(self) -> Result<()> {
        info!("Fetching Azure resource groups...");
        let resource_groups = fetch_all_resource_groups().await?;
        info!(
            count = resource_groups.len(),
            "Fetched Azure resource groups"
        );

        let chosen = PickerTui::new().pick_many(resource_groups)?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &chosen)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
