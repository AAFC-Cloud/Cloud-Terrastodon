use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_azure::AzureTenantAlias;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::remove_tracked_tenant_aliases;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use facet::Facet;

/// Arguments for removing aliases from a tracked Azure tenant.
#[derive(Facet, Debug, Clone)]
pub struct AzureTenantAliasRemoveArgs {
    /// Tracked tenant id or Cloud Terrastodon alias.
    #[facet(figue::named)]
    pub tenant: AzureTenantArgument<'static>,

    /// One or more aliases to remove from the tracked tenant.
    #[facet(figue::positional)]
    pub aliases: Vec<AzureTenantAlias>,
}

impl AzureTenantAliasRemoveArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        let tenant_id = self.tenant.resolve(cancellation).await?;
        let aliases = remove_tracked_tenant_aliases(tenant_id, &self.aliases).await?;

        Ok(CliOutput::facet(aliases))
    }
}
