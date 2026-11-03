use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth::logout_all_auth_sources;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;

/// Log out of all tenants and organizations across stored authentication sources.
/// Clears browser/PIM refresh tokens, runs `az logout`, `az account clear`, and `az devops logout`,
/// and attempts every source even if another fails. Environment credentials
/// must be removed from the calling shell or pipeline separately.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureTenantLogoutArgs;

impl AzureTenantLogoutArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        logout_all_auth_sources(cancellation).await?;
        Ok(CliOutput::facet_with_text(
            "Logged out of stored browser/PIM, Azure CLI, and Azure DevOps credentials. Environment-supplied credentials must be removed from the calling shell or pipeline separately.".to_owned(),
            |message, writer, _cancellation| {
                write!(writer, "{message}")?;
                Ok(())
            },
        ))
    }
}
