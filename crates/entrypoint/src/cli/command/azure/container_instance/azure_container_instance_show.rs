use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::fetch_all_container_instances;
use eyre::Result;
use eyre::bail;
use std::io::Write;
use tracing::info;

/// Arguments for showing a single Azure Container Instance container group.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureContainerInstanceShowArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,

    /// Container Instance resource id, resource name, or private IP address.
    #[facet(figue::positional)]
    pub container_instance: String,
}

impl AzureContainerInstanceShowArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!(
            needle = %self.container_instance,
            %tenant_id,
            "Fetching Azure container instances"
        );
        let container_instances = fetch_all_container_instances(tenant_id).await?;
        info!(
            count = container_instances.len(),
            "Fetched Azure container instances"
        );

        let needle = self.container_instance.trim();
        let mut matches = container_instances
            .into_iter()
            .filter(|container_instance| {
                container_instance.id.expanded_form() == needle
                    || container_instance.name.eq_ignore_ascii_case(needle)
                    || container_instance
                        .properties
                        .ip_address
                        .as_ref()
                        .and_then(|address| address.ip)
                        .map(|ip| ip.to_string() == needle)
                        .unwrap_or(false)
            })
            .collect::<Vec<_>>();

        match matches.len() {
            0 => bail!("No container instance found matching '{}'.", needle),
            1 => {
                let stdout = std::io::stdout();
                let mut handle = stdout.lock();
                cloud_terrastodon_command::to_writer_pretty(&mut handle, &matches.remove(0))?;
                handle.write_all(b"\n")?;
                Ok(())
            }
            _ => {
                matches.sort_by_key(|container_instance| container_instance.id.expanded_form());
                let ids = matches
                    .iter()
                    .map(|container_instance| container_instance.id.expanded_form())
                    .collect::<Vec<_>>()
                    .join("\n  ");
                bail!(
                    "Multiple container instances matched '{}'. Use a full resource id.\n  {}",
                    needle,
                    ids
                )
            }
        }
    }
}
