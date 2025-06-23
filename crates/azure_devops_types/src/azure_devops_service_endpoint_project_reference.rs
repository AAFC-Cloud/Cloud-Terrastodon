use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AzureDevOpsServiceEndpointProjectReference {
    #[serde(default)]
    pub description: String,
    pub name: String,
    pub project_reference: AzureDevOpsServiceEndpointProjectReferenceProjectReference,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AzureDevOpsServiceEndpointProjectReferenceProjectReference {
    pub id: Uuid,
    pub name: String,
}
