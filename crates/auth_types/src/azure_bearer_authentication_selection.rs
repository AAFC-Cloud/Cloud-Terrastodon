use crate::AuthenticationSelection;
use crate::AzureBearerAuthSource;
use crate::AzureDevOpsAuthSelection;
use crate::CloudTerrastodonAccountId;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::AzureTenantId;
use facet::Facet;

/// Reviewed account, credential source, and explicit Entra token authority.
///
/// Credentials and token caches belong to the executor's authentication provider.
/// See [Entra token authorities](https://learn.microsoft.com/en-us/entra/identity-platform/authentication-national-cloud).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Arbitrary, Facet)]
pub struct AzureBearerAuthenticationSelection {
    pub account_id: CloudTerrastodonAccountId,
    pub auth_source: AzureBearerAuthSource,
    pub tenant_id: AzureTenantId,
}

impl From<AzureBearerAuthenticationSelection> for AuthenticationSelection {
    fn from(value: AzureBearerAuthenticationSelection) -> Self {
        Self::AzureBearer(value)
    }
}

impl From<AzureBearerAuthenticationSelection> for AzureDevOpsAuthSelection {
    fn from(value: AzureBearerAuthenticationSelection) -> Self {
        Self::Bearer(value)
    }
}

cloud_terrastodon_registry::register_thing!(AzureBearerAuthenticationSelection);
cloud_terrastodon_registry::register_arbitrary!(AzureBearerAuthenticationSelection);
cloud_terrastodon_registry::register_from!(AzureBearerAuthenticationSelection => AuthenticationSelection);
cloud_terrastodon_registry::register_from!(AzureBearerAuthenticationSelection => AzureDevOpsAuthSelection);
