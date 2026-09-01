use cloud_terrastodon_azure::AzurePrincipalArgument;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_principals;
use cloud_terrastodon_azure::fetch_principal;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use eyre::bail;
use std::io::Write;
use tracing::info;

/// Show a single Entra (Azure AD) principal.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraPrincipalShowArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,

    /// Principal object id, display name, or user principal name.
    #[facet(figue::positional)]
    pub principal: AzurePrincipalArgument<'static>,
}

impl AzureEntraPrincipalShowArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        let tenant_auth_context = self.tenant.bind_auth_context(auth_context).await?;
        info!(needle = %self.principal, tenant_id = %tenant_auth_context.tenant_id, "Fetching Entra principal");

        if let Some(principal_id) = self.principal.as_id() {
            let principal = fetch_principal(*principal_id, &tenant_auth_context).await?;
            let stdout = std::io::stdout();
            let mut handle = stdout.lock();
            cloud_terrastodon_command::to_writer_pretty(&mut handle, &principal)?;
            handle.write_all(b"\n")?;
            return Ok(());
        }

        let principals = fetch_all_principals(&tenant_auth_context).await?;
        let mut matches = principals
            .values()
            .filter(|principal| self.principal.matches(*principal))
            .collect::<Vec<_>>();

        match matches.len() {
            0 => bail!("No principal found matching '{}'.", self.principal),
            1 => {
                let stdout = std::io::stdout();
                let mut handle = stdout.lock();
                cloud_terrastodon_command::to_writer_pretty(&mut handle, matches.remove(0))?;
                handle.write_all(b"\n")?;
                Ok(())
            }
            _ => {
                matches.sort_by_key(|principal| principal.id().to_string());
                let ids = matches
                    .iter()
                    .map(|principal| principal.id().to_string())
                    .collect::<Vec<_>>()
                    .join("\n  ");
                bail!(
                    "Multiple principals matched '{}'. Use a full object id.\n  {}",
                    self.principal,
                    ids
                )
            }
        }
    }
}
