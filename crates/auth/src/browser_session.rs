use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraApplicationClientId;
use facet::Facet;

/// A saved delegated refresh credential and the tenant/client that issued it.
///
/// The runtime snapshots this credential when creating an authentication context.
/// Token acquisition never reloads it to substitute a different signed-in user.
/// See the [Entra refresh-token protocol](https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-auth-code-flow#refresh-the-access-token).
#[derive(Clone, Facet)]
pub struct BrowserSession {
    pub(crate) tenant_id: AzureTenantId,
    pub(crate) client_id: EntraApplicationClientId,
    #[facet(sensitive)]
    pub(crate) refresh_token: String,
}

impl std::fmt::Debug for BrowserSession {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BrowserSession")
            .field("tenant_id", &self.tenant_id)
            .field("client_id", &self.client_id)
            .field("refresh_token", &"***redacted***")
            .finish()
    }
}
