use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraApplicationClientId;
use eyre::Context;
use eyre::Result;
use facet::Facet;
use reqwest::Url;
use std::fmt::Debug;

const AZURE_TOKEN_ENDPOINT_TEMPLATE: &str =
    "https://login.microsoftonline.com/{tenant}/oauth2/v2.0/token";

/// Normalized Azure DevOps/Entra workload identity inputs.
#[derive(Clone, Facet)]
pub struct WorkloadIdentityConfig {
    pub client_id: EntraApplicationClientId,
    pub tenant_id: AzureTenantId,
    #[facet(sensitive)]
    pub federated_token: String,
}

impl WorkloadIdentityConfig {
    pub fn federated_token(&self) -> &str {
        &self.federated_token
    }

    pub fn token_endpoint(&self) -> Result<Url> {
        Url::parse(
            AZURE_TOKEN_ENDPOINT_TEMPLATE
                .replace("{tenant}", &self.tenant_id.to_string())
                .as_str(),
        )
        .wrap_err("building the Entra workload-identity token endpoint")
    }
}

impl Debug for WorkloadIdentityConfig {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorkloadIdentityConfig")
            .field("client_id", &self.client_id)
            .field("tenant_id", &self.tenant_id)
            .field("federated_token", &"***redacted***")
            .finish()
    }
}
