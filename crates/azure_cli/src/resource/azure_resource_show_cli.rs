use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::fetch_all_resources;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use std::io::Write;
use tracing::info;

/// Arguments for showing a single Azure resource by id or by name.
#[derive(Facet, Debug, Clone)]
pub struct AzureResourceShowArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,

    /// Resource id (scope) or resource name.
    #[facet(figue::positional)]
    pub resource: String,
}

impl AzureResourceShowArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        let cancellation = executor.cancellation();
        let tenant_auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        let executor = &tenant_auth_context.auth_context.bind_executor(executor)?;
        info!(needle = %self.resource, "Fetching all Azure resources");
        let resources = fetch_all_resources(&tenant_auth_context, executor).await?;
        info!(count = resources.len(), "Fetched Azure resources");

        if let Some(resource) = resources
            .iter()
            .find(|resource| resource.id.expanded_form() == self.resource)
        {
            let stdout = std::io::stdout();
            let mut handle = stdout.lock();
            facet_json::to_writer_std_pretty(&mut handle, resource)
                .map_err(|error| eyre::eyre!("{error:?}"))?;
            handle.write_all(b"\n")?;
            return Ok(());
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
            1 => {
                let stdout = std::io::stdout();
                let mut handle = stdout.lock();
                facet_json::to_writer_std_pretty(&mut handle, &named_matches.remove(0))
                    .map_err(|error| eyre::eyre!("{error:?}"))?;
                handle.write_all(b"\n")?;
                Ok(())
            }
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
