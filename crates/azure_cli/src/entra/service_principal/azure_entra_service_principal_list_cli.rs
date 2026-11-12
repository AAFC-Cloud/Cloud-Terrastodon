use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::EntraServicePrincipalListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use tracing::info;

/// List Entra (Azure AD) service principals.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraSpListArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl AzureEntraSpListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        info!("Fetching service principals");
        let sps = EntraServicePrincipalListRequest { authentication }
            .execute(executor)
            .await?;
        Ok(CliOutput::facet_with_text(
            sps,
            |principals, writer, cancellation| {
                for sp in principals.iter() {
                    cancellation.bail_if_cancelled()?;
                    writeln!(
                        writer,
                        "{} {:64} {} {}",
                        sp.id,
                        sp.display_name,
                        sp.app_id,
                        sp.service_principal_names.join(",")
                    )?;
                }
                Ok(())
            },
        ))
    }
}
