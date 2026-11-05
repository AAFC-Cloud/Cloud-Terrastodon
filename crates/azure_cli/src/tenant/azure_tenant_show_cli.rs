use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_azure_tenant_details;
use cloud_terrastodon_command::CancellationScope;
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
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        let tenant_auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        let details = fetch_azure_tenant_details(&tenant_auth_context, cancellation).await?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        facet_json::to_writer_std_pretty(&mut handle, &details)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
