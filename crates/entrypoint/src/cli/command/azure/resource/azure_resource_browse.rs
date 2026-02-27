use clap::Args;
use cloud_terrastodon_azure::prelude::Resource;
use cloud_terrastodon_azure::prelude::Scope;
use cloud_terrastodon_azure::prelude::fetch_all_resources;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Arguments for browsing Azure resources interactively.
#[derive(Args, Debug, Clone)]
pub struct AzureResourceBrowseArgs {}

impl AzureResourceBrowseArgs {
    pub async fn invoke(self) -> Result<()> {
        info!("Fetching Azure resources...");
        let resources = fetch_all_resources().await?;
        info!(count = resources.len(), "Fetched Azure resources");

        let choices = resources.into_iter().map(|resource| Choice {
            key: format!("{} - {}", resource.name, resource.id.expanded_form()),
            value: resource,
        });

        let chosen: Vec<Resource> = PickerTui::new()
            .set_header("Select Azure resources")
            .pick_many(choices)?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &chosen)?;
        handle.write_all(b"\n")?;

        Ok(())
    }
}
