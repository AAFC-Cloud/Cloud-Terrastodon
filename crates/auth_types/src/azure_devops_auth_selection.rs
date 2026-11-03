use crate::AzureBearerAuthSource;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::AzureTenantId;
use facet::Facet;

/// Credential selection carried by a serializable Azure DevOps request.
///
/// Bearer acquisition requires an explicit tenant, including Azure CLI. PAT
/// selection is tenant-independent; the execution provider supplies the secret.
/// Organization and project scope remain properties of the service request.
/// See [Azure DevOps authentication methods](https://learn.microsoft.com/en-us/azure/devops/integrate/get-started/authentication/authentication-guidance?view=azure-devops).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Arbitrary, Facet)]
#[repr(u8)]
pub enum AzureDevOpsAuthSelection {
    Bearer {
        auth_source: AzureBearerAuthSource,
        tenant_id: AzureTenantId,
    },
    PersonalAccessToken,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsAuthSelection);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsAuthSelection);
