use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureResourceListRequest;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use std::sync::Arc;
use tracing::info;

/// Arguments for showing a single Azure resource by id or by name.
#[derive(Facet, Debug, Clone)]
pub struct AzureResourceShowArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// Resource id (scope) or resource name.
    #[facet(figue::positional)]
    pub resource: String,
}

impl AzureResourceShowArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        info!(needle = %self.resource, "Fetching all Azure resources");
        let resources = AzureResourceListRequest { authentication }
            .execute(executor)
            .await?;
        info!(count = resources.len(), "Fetched Azure resources");

        if let Some(resource) = resources
            .iter()
            .find(|resource| resource.id.expanded_form() == self.resource)
        {
            return Ok(CliOutput::facet(Arc::clone(resource)));
        }

        let mut named_matches = resources
            .into_iter()
            .filter(|resource| {
                resource.name == self.resource
                    || resource.display_name().is_some_and(|x| x == self.resource)
            })
            .collect::<Vec<_>>();

        match named_matches.len() {
            0 => {
                bail!("No resource found matching '{}'.", self.resource);
            }
            1 => Ok(CliOutput::facet(named_matches.remove(0))),
            _ => {
                named_matches.sort_by_key(|a| a.id.expanded_form());
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
