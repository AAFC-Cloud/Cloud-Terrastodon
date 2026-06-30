use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::Resource;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::fetch_all_resources;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Arguments for browsing Azure resources interactively.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureResourceBrowseArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureResourceBrowseArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!("Fetching Azure resources...");
        let resources = fetch_all_resources(tenant_id).await?;
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
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &chosen)?;
        handle.write_all(b"\n")?;

        Ok(())
    }
}
