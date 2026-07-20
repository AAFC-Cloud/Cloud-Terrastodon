use cloud_terrastodon_azure::AzurePrincipalArgument;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_entra_user;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Show a single Entra (Azure AD) user.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraUserShowArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,

    /// User object id or user principal name.
    #[facet(figue::positional)]
    pub user: AzurePrincipalArgument<'static>,
}

impl AzureEntraUserShowArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!(needle = %self.user, %tenant_id, "Fetching user");

        let user = fetch_entra_user(tenant_id, self.user).await?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &user)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
