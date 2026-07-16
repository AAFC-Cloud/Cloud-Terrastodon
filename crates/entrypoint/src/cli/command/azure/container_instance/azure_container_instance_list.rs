use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_container_instances;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Arguments for listing Azure Container Instance container groups.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureContainerInstanceListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureContainerInstanceListArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!(%tenant_id, "Fetching Azure container instances");
        let container_instances = fetch_all_container_instances(tenant_id).await?;
        info!(
            count = container_instances.len(),
            "Fetched Azure container instances"
        );

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &container_instances)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
