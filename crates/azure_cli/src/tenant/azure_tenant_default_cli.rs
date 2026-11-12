use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::set_default_tenant_id;
use cloud_terrastodon_command::CancellationScope;
use facet::Facet;

/// Configure the Cloud Terrastodon tenant used by an explicit default selector.
#[derive(Debug, Clone, Facet)]
pub struct AzureTenantDefaultArgs {
    /// Tracked tenant ID or alias. This never changes the active Azure CLI account.
    #[facet(figue::positional)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureTenantDefaultArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> eyre::Result<CliOutput> {
        let tenant = self.tenant.resolve(cancellation).await?;
        set_default_tenant_id(tenant, cancellation).await?;
        Ok(CliOutput::facet(tenant))
    }
}
