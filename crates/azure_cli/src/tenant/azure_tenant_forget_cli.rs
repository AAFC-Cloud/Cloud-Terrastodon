use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::forget_tracked_tenant;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use eyre::bail;
use facet::Facet;

/// Arguments for forgetting a tracked Azure tenant.
#[derive(Facet, Debug, Clone)]
pub struct AzureTenantForgetArgs {
    /// Tenant id (GUID) or alias to forget.
    #[facet(figue::positional)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureTenantForgetArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        let tenant_id = self.tenant.resolve(cancellation).await?;
        let Some(tenant_id) = forget_tracked_tenant(tenant_id).await? else {
            bail!("Tracked tenant '{}' was not found.", tenant_id);
        };

        Ok(CliOutput::facet(tenant_id))
    }
}
