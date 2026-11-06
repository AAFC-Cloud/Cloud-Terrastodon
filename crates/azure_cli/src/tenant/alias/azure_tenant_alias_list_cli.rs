use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::list_tracked_tenant_aliases;
use cloud_terrastodon_azure::list_tracked_tenant_aliases_for;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use facet::Facet;
use std::io::Write;

/// Arguments for listing aliases for tracked Azure tenants.
#[derive(Facet, Debug, Clone)]
pub struct AzureTenantAliasListArgs {
    /// Optional tracked tenant id or Cloud Terrastodon alias to filter by.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,
}

impl AzureTenantAliasListArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<()> {
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();

        if let Some(tenant) = self.tenant {
            let tenant_id = tenant.resolve(cancellation).await?;
            let mut aliases = list_tracked_tenant_aliases_for(tenant_id).await?;
            aliases.sort();
            facet_json::to_writer_std_pretty(&mut handle, &aliases)
                .map_err(|error| eyre::eyre!("{error:?}"))?;
        } else {
            let aliases = list_tracked_tenant_aliases().await?;
            facet_json::to_writer_std_pretty(&mut handle, &aliases)
                .map_err(|error| eyre::eyre!("{error:?}"))?;
        }

        handle.write_all(b"\n")?;
        Ok(())
    }
}
