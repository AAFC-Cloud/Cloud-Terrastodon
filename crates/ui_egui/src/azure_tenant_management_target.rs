use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::AzureTenantId;

/// Distinguishes local tenant operations from account-bound directory lookups.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum AzureTenantManagementTarget {
    Catalogue,
    Tenant(AzureTenantId),
    Directory {
        tenant: AzureTenantId,
        selected: Option<AzureBearerAuthenticationSelection>,
    },
}
