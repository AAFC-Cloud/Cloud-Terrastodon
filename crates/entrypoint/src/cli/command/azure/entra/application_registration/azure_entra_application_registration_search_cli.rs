use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::search_application_registrations;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Search Entra (Azure AD) application registrations by app id, name, or URI prefix.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraApplicationRegistrationSearchArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,

    /// Search text. Graph filters application identifiers and names server-side.
    #[facet(figue::positional)]
    pub search_term: String,
}

impl AzureEntraApplicationRegistrationSearchArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        let tenant_auth_context = self.tenant.bind_auth_context(auth_context).await?;
        info!(tenant_id = %tenant_auth_context.tenant_id, search_term = %self.search_term, "Searching application registrations");
        let applications =
            search_application_registrations(self.search_term, &tenant_auth_context).await?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &applications)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
