use clap::Args;
use cloud_terrastodon_azure::prelude::Scope;
use cloud_terrastodon_azure::prelude::fetch_all_resources;
use eyre::Result;
use eyre::bail;
use std::io::Write;
use tracing::info;

/// Arguments for showing a single Azure resource by id or by name.
#[derive(Args, Debug, Clone)]
pub struct AzureResourceShowArgs {
    /// Resource id (scope) or resource name.
    pub resource: String,
}

impl AzureResourceShowArgs {
    pub async fn invoke(self) -> Result<()> {
        info!(needle = %self.resource, "Fetching all Azure resources");
        let resources = fetch_all_resources().await?;
        info!(count = resources.len(), "Fetched Azure resources");

        if let Some(resource) = resources
            .iter()
            .find(|resource| resource.id.expanded_form() == self.resource)
        {
            let stdout = std::io::stdout();
            let mut handle = stdout.lock();
            serde_json::to_writer_pretty(&mut handle, resource)?;
            handle.write_all(b"\n")?;
            return Ok(());
        }

        let mut named_matches = resources
            .into_iter()
            .filter(|resource| {
                resource.name == self.resource
                    || resource
                        .display_name
                        .as_ref()
                        .map(|x| x == &self.resource)
                        .unwrap_or(false)
            })
            .collect::<Vec<_>>();

        match named_matches.len() {
            0 => {
                bail!("No resource found matching '{}'.", self.resource);
            }
            1 => {
                let stdout = std::io::stdout();
                let mut handle = stdout.lock();
                serde_json::to_writer_pretty(&mut handle, &named_matches.remove(0))?;
                handle.write_all(b"\n")?;
                Ok(())
            }
            _ => {
                named_matches.sort_by(|a, b| a.id.expanded_form().cmp(&b.id.expanded_form()));
                let ids = named_matches
                    .iter()
                    .map(|resource| resource.id.expanded_form())
                    .collect::<Vec<_>>()
                    .join("\n  ");
                bail!(
                    "Multiple resources matched '{}'. Use a full resource id.\n  {}",
                    self.resource,
                    ids
                );
            }
        }
    }
}
