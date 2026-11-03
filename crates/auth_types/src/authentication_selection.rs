use crate::AzureBearerAuthSource;
use crate::AzureDevOpsAuthSelection;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::AzureTenantId;
use facet::Facet;

/// Non-secret authentication intent admitted by an execution provider.
///
/// This is request data rather than acquired authorization or a credential-cache
/// identity. Anonymous execution requests no credential discovery. Bearer
/// selections always carry a resolved source and an explicit tenant.
/// See [Microsoft identity scopes and resource audiences](https://learn.microsoft.com/en-us/entra/identity-platform/scopes-oidc).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Arbitrary, Facet)]
#[repr(u8)]
pub enum AuthenticationSelection {
    Anonymous,
    AzureBearer {
        auth_source: AzureBearerAuthSource,
        tenant_id: AzureTenantId,
    },
    AzureDevOpsPersonalAccessToken,
}

impl From<AzureDevOpsAuthSelection> for AuthenticationSelection {
    fn from(selection: AzureDevOpsAuthSelection) -> Self {
        match selection {
            AzureDevOpsAuthSelection::Bearer {
                auth_source,
                tenant_id,
            } => Self::AzureBearer {
                auth_source,
                tenant_id,
            },
            AzureDevOpsAuthSelection::PersonalAccessToken => Self::AzureDevOpsPersonalAccessToken,
        }
    }
}

cloud_terrastodon_registry::register_thing!(AuthenticationSelection);
cloud_terrastodon_registry::register_arbitrary!(AuthenticationSelection);
cloud_terrastodon_registry::register_from!(AzureDevOpsAuthSelection => AuthenticationSelection);
