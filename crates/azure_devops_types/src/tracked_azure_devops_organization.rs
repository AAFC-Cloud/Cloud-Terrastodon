use crate::AzureDevOpsOrganizationUrl;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::AzureTenantId;
use facet::Facet;

/// A locally tracked Azure DevOps organization or Server collection.
///
/// The optional tenant is recorded by the user. See [Connect your organization
/// to Microsoft Entra ID](https://learn.microsoft.com/en-us/azure/devops/organizations/accounts/connect-organization-to-azure-ad?view=azure-devops).
#[derive(Arbitrary, Clone, Debug, Eq, Facet, PartialEq)]
pub struct TrackedAzureDevOpsOrganization {
    pub url: AzureDevOpsOrganizationUrl,
    #[facet(default)]
    pub tenant_id: Option<AzureTenantId>,
}

cloud_terrastodon_registry::register_thing!(TrackedAzureDevOpsOrganization);
cloud_terrastodon_registry::register_arbitrary!(TrackedAzureDevOpsOrganization);
