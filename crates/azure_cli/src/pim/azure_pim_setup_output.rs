use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure::EntraApplicationClientId;
use facet::Facet;

/// Inspected delegated PIM application and explicit account-registration guidance.
#[derive(Debug, Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzurePimSetupOutput {
    pub tenant_id: AzureTenantId,
    pub application_id: EntraApplicationClientId,
    pub application_display_name: String,
    pub configured_graph_scopes: Vec<String>,
    pub protocol_scopes: Vec<String>,
    pub account_registration_command: String,
}
