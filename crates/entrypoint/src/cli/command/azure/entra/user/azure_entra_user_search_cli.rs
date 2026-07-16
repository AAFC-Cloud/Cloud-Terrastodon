use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::search_entra_users;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Search Entra (Azure AD) users by name, email address, or user principal name.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraUserSearchArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,

    /// Search text. Matching is performed against the beginning of user name fields.
    #[facet(figue::positional)]
    pub search_term: String,
}

impl AzureEntraUserSearchArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!(%tenant_id, search_term = %self.search_term, "Searching Entra users");
        let users = search_entra_users(tenant_id, self.search_term).await?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &users)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
