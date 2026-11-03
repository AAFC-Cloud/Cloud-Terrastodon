use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraApplicationClientId;
use facet::Facet;

/// Nonsecret lookup metadata for a refresh credential held in Windows Credential Manager.
#[derive(Clone, Facet)]
pub(crate) struct BrowserSessionMetadata {
    pub(crate) tenant_id: AzureTenantId,
    pub(crate) client_id: EntraApplicationClientId,
}
