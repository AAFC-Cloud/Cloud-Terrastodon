use crate::AzureDevOpsEntraPermissionIdentity;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use facet::Facet;

/// The outcome of correlating an Entra principal in one Azure DevOps organization.
#[derive(Debug, Facet)]
pub struct AzureDevOpsEntraPermissionOrganization<Failure> {
    pub organization: AzureDevOpsOrganizationUrl,
    pub result: Result<Vec<AzureDevOpsEntraPermissionIdentity<Failure>>, Failure>,
}

impl<Failure> AzureDevOpsEntraPermissionOrganization<Failure> {
    /// Convert failures for presentation without discarding partial results.
    pub fn map_failure<Mapped>(
        self,
        mut map: impl FnMut(Failure) -> Mapped,
    ) -> AzureDevOpsEntraPermissionOrganization<Mapped> {
        AzureDevOpsEntraPermissionOrganization {
            organization: self.organization,
            result: match self.result {
                Err(error) => Err(map(error)),
                Ok(identities) => Ok(identities
                    .into_iter()
                    .map(|identity| AzureDevOpsEntraPermissionIdentity {
                        display_name: identity.display_name,
                        descriptor: identity.descriptor,
                        result: identity.result.map_err(&mut map),
                    })
                    .collect()),
            },
        }
    }
}
