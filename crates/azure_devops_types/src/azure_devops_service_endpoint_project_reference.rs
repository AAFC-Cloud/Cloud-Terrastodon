use crate::prelude::AzureDevOpsProjectId;
use crate::prelude::AzureDevOpsProjectName;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AzureDevOpsServiceEndpointProjectReference {
    #[serde(default)]
    pub description: String,
    pub name: AzureDevOpsProjectName,
    pub project_reference: AzureDevOpsServiceEndpointProjectReferenceProjectReference,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AzureDevOpsServiceEndpointProjectReferenceProjectReference {
    pub id: AzureDevOpsProjectId,
    pub name: AzureDevOpsProjectName,
}
