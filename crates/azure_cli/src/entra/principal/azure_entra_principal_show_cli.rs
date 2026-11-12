use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzurePrincipalArgument;
use cloud_terrastodon_azure::EntraPrincipalListRequest;
use cloud_terrastodon_azure::EntraPrincipalRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use tracing::info;

/// Show a single Entra (Azure AD) principal.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraPrincipalShowArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// Principal object id, display name, or user principal name.
    #[facet(figue::positional)]
    pub principal: AzurePrincipalArgument<'static>,
}

impl AzureEntraPrincipalShowArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        info!(needle = %self.principal, tenant_id = %authentication.tenant_id, "Fetching Entra principal");

        if let Some(principal_id) = self.principal.as_id() {
            let principal = EntraPrincipalRequest {
                lookup: (*principal_id).into(),
                authentication,
            }
            .execute(executor)
            .await?;
            return Ok(CliOutput::facet(principal));
        }

        let principals = EntraPrincipalListRequest { authentication }
            .execute(executor)
            .await?;
        let mut matches = principals
            .values()
            .filter(|principal| self.principal.matches(*principal))
            .collect::<Vec<_>>();

        match matches.len() {
            0 => {
                bail!("No principal found matching '{}'.", self.principal);
            }
            1 => Ok(CliOutput::facet(matches.remove(0).clone())),
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
                );
            }
        }
    }
}
