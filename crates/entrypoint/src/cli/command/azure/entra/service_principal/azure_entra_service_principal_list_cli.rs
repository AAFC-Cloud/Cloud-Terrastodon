use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_service_principals;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// List Entra (Azure AD) service principals.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraSpListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureEntraSpListArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        let tenant_auth_context = self.tenant.bind_auth_context(auth_context).await?;
        info!("Fetching service principals");
        let sps = fetch_all_service_principals(&tenant_auth_context).await?;
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        for sp in sps {
            writeln!(
                out,
                "{} {:64} {} {}",
                sp.id,
                sp.display_name,
                sp.app_id,
                sp.service_principal_names.join(",")
            )?;
        }
        Ok(())
    }
}
