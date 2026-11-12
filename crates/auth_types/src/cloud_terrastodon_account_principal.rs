use cloud_terrastodon_azure_types::EntraApplicationClientId;
use cloud_terrastodon_azure_types::EntraUserId;
use facet::Facet;

/// Identity established by successful tenant-bound credential provisioning.
/// The containing account owns the tenant; mechanisms attached to it must
/// establish the same identity before their sessions become active.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Facet)]
#[repr(u8)]
pub enum CloudTerrastodonAccountPrincipal {
    /// Tenant-local user identity returned by authenticated Graph `/me`.
    /// <https://learn.microsoft.com/en-us/graph/api/user-get?view=graph-rest-1.0>
    EntraUser(EntraUserId),
    /// Application identity authenticated by the tenant's token endpoint.
    /// <https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-client-creds-grant-flow>
    EntraApplication(EntraApplicationClientId),
}
cloud_terrastodon_registry::register_thing!(CloudTerrastodonAccountPrincipal);
