use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureContainerInstanceListRequest;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use tracing::info;

/// Arguments for showing a single Azure Container Instance container group.
#[derive(Facet, Debug, Clone)]
pub struct AzureContainerInstanceShowArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// Container Instance resource id, resource name, IP address, or FQDN.
    #[facet(figue::positional)]
    pub container_instance: String,
}

impl AzureContainerInstanceShowArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        info!(
            needle = %self.container_instance,
            tenant_id = %authentication.tenant_id,
            "Fetching Azure container instances"
        );
        let container_instances = AzureContainerInstanceListRequest { authentication }
            .execute(executor)
            .await?;
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
                    || container_instance
                        .properties
                        .ip_address
                        .as_ref()
                        .and_then(|address| address.fqdn.as_deref())
                        .map(|fqdn| fqdn.eq_ignore_ascii_case(needle))
                        .unwrap_or(false)
            })
            .collect::<Vec<_>>();

        match matches.len() {
            0 => {
                bail!("No container instance found matching '{}'.", needle);
            }
            1 => Ok(CliOutput::facet(matches.remove(0))),
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
                );
            }
        }
    }
}
