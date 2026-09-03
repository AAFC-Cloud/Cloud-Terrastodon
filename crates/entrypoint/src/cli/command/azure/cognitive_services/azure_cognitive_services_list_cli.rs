use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_cognitive_services_accounts;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use std::io::Write;

/// Arguments for listing Azure Cognitive Services accounts.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureCognitiveServicesListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureCognitiveServicesListArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        let tenant_auth_context = self.tenant.bind_auth_context(auth_context).await?;
        let accounts = fetch_all_cognitive_services_accounts(&tenant_auth_context).await?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &accounts)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
