use cloud_terrastodon_azure::AzurePrincipalArgument;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_entra_users;
use eyre::Result;
use eyre::bail;
use std::io::Write;
use tracing::info;

/// Show a single Entra (Azure AD) user.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraUserShowArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,

    /// User object id, user principal name, display name, or email address.
    pub user: AzurePrincipalArgument<'static>,
}

impl AzureEntraUserShowArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!(needle = %self.user, %tenant_id, "Fetching users");
        let users = fetch_all_entra_users(tenant_id).await?;
        info!(count = users.len(), "Fetched users");

        let mut matches = users
            .into_iter()
            .filter(|user| self.user.matches(user))
            .collect::<Vec<_>>();

        match matches.len() {
            0 => bail!("No user found matching '{}'.", self.user),
            1 => {
                let stdout = std::io::stdout();
                let mut handle = stdout.lock();
                cloud_terrastodon_command::to_writer_pretty(&mut handle, &matches.remove(0))?;
                handle.write_all(b"\n")?;
                Ok(())
            }
            _ => {
                matches.sort_by_key(|user| user.id.to_string());
                let ids = matches
                    .iter()
                    .map(|user| user.id.to_string())
                    .collect::<Vec<_>>()
                    .join("\n  ");
                bail!(
                    "Multiple users matched '{}'. Use a full object id.\n  {}",
                    self.user,
                    ids
                )
            }
        }
    }
}
