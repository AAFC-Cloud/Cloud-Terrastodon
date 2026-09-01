use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_azure_tenant_details;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use std::io::Write;

/// Arguments for showing a tracked Azure tenant.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureTenantShowArgs {
    /// Tenant id (GUID) or alias to show.
    #[facet(figue::positional)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureTenantShowArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        let tenant_auth_context = self.tenant.bind_auth_context(auth_context).await?;
        let details = fetch_azure_tenant_details(&tenant_auth_context).await?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &details)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
