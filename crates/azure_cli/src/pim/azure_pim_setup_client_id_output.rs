use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure::EntraApplicationClientId;
use facet::Facet;

/// Explicit application selection without a live permission inspection.
#[derive(Debug, Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzurePimSetupClientIdOutput {
    pub tenant_id: AzureTenantId,
    pub application_id: EntraApplicationClientId,
    pub permissions_verified: bool,
    /// Suggested command; replace `account-alias` with the desired account name.
    pub account_registration_command: String,
}
