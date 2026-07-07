use crate::AzureDevOpsProjectId;
use crate::AzureDevOpsProjectName;
use crate::AzureDevOpsServiceEndpointName;
use arbitrary::Arbitrary;

#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsServiceEndpointProjectReference {
    #[facet(default)]
    pub description: String,
    pub name: AzureDevOpsServiceEndpointName,
    pub project_reference: AzureDevOpsServiceEndpointProjectReferenceProjectReference,
}

#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsServiceEndpointProjectReferenceProjectReference {
    pub id: AzureDevOpsProjectId,
    /// Name may be missing if the project has been deleted
    pub name: Option<AzureDevOpsProjectName>,
}
