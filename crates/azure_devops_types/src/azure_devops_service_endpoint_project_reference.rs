use crate::prelude::AzureDevOpsProjectId;
use crate::prelude::AzureDevOpsProjectName;
use crate::prelude::AzureDevOpsServiceEndpointName;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AzureDevOpsServiceEndpointProjectReference {
    #[serde(default)]
    pub description: String,
    pub name: AzureDevOpsServiceEndpointName,
    pub project_reference: AzureDevOpsServiceEndpointProjectReferenceProjectReference,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AzureDevOpsServiceEndpointProjectReferenceProjectReference {
    pub id: AzureDevOpsProjectId,
    /// Name may be missing if the project has been deleted
    pub name: Option<AzureDevOpsProjectName>,
}
