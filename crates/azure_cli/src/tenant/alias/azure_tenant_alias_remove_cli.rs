use cloud_terrastodon_azure::AzureTenantAlias;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::remove_tracked_tenant_aliases;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use std::io::Write;

/// Arguments for removing aliases from a tracked Azure tenant.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureTenantAliasRemoveArgs {
    /// Tracked tenant id or Cloud Terrastodon alias.
    #[facet(figue::named)]
    pub tenant: AzureTenantArgument<'static>,

    /// One or more aliases to remove from the tracked tenant.
    #[facet(figue::positional)]
    pub aliases: Vec<AzureTenantAlias>,
}

impl AzureTenantAliasRemoveArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<()> {
        let tenant_id = self.tenant.resolve(cancellation).await?;
        let aliases = remove_tracked_tenant_aliases(tenant_id, &self.aliases).await?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        facet_json::to_writer_std_pretty(&mut handle, &aliases)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
