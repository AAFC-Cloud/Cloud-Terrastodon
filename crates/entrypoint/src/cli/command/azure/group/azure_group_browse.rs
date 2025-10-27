use clap::Args;
use cloud_terrastodon_azure::prelude::fetch_all_resource_groups;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use std::io::Write;

/// Arguments for browsing Azure resource groups.
#[derive(Args, Debug, Clone)]
pub struct AzureGroupBrowseArgs {}

impl AzureGroupBrowseArgs {
    pub async fn invoke(self) -> Result<()> {
        let resource_groups = fetch_all_resource_groups().await?;
        let chosen = PickerTui::new(resource_groups).pick_many()?;
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &chosen)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
