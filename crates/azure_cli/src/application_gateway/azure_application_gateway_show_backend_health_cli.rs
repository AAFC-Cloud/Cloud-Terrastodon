use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureApplicationGatewayListRequest;
use cloud_terrastodon_azure::AzureApplicationGatewayResourceBackendHealthRequest;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use tracing::info;

/// Arguments for showing backend health for a single Azure application gateway.
#[derive(Facet, Debug, Clone)]
pub struct AzureApplicationGatewayShowBackendHealthArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// Application gateway resource id or resource name.
    #[facet(figue::positional)]
    pub application_gateway: String,
}

impl AzureApplicationGatewayShowBackendHealthArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        let tenant_id = authentication.tenant_id;
        info!(needle = %self.application_gateway, %tenant_id, "Resolving Azure application gateway for backend health");
        let application_gateways = AzureApplicationGatewayListRequest { authentication }
            .execute(executor)
            .await?;

        let needle = self.application_gateway.trim();
        let mut matches = application_gateways
            .into_iter()
            .filter(|application_gateway| {
                application_gateway.id.expanded_form() == needle
                    || application_gateway.name.eq_ignore_ascii_case(needle)
            })
            .collect::<Vec<_>>();

        match matches.len() {
            0 => {
                bail!("No application gateway found matching '{}'.", needle);
            }
            1 => {
                let application_gateway = matches.remove(0);
                let backend_health = AzureApplicationGatewayResourceBackendHealthRequest {
                    authentication,
                    application_gateway_id: (application_gateway.id.clone()),
                }
                .execute(executor)
                .await?;
                Ok(CliOutput::facet(backend_health))
            }
            _ => {
                matches.sort_by_key(|application_gateway| application_gateway.id.expanded_form());
                let ids = matches
                    .iter()
                    .map(|application_gateway| application_gateway.id.expanded_form())
                    .collect::<Vec<_>>()
                    .join("\n  ");
                bail!(
                    "Multiple application gateways matched '{}'. Use a full resource id.\n  {}",
                    needle,
                    ids
                );
            }
        }
    }
}
