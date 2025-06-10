use serde_json::Value;

use crate::prelude::AzureDevOpsServiceEndpointId;
use crate::prelude::AzureDevOpsServiceEndpointName;

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AzureDevOpsServiceEndpoint {
    pub administrators_group: Option<Value>,
    pub authorization: Value,
    pub created_by: Value,
    pub data: Value,
    pub description: Value,
    pub group_scope_id: Option<Value>,
    pub id: AzureDevOpsServiceEndpointId,
    pub is_outdated: Value,
    pub is_ready: Value,
    pub is_shared: Value,
    pub name: AzureDevOpsServiceEndpointName,
    pub operation_status: Option<Value>,
    pub owner: Value,
    pub readers_group: Option<Value>,
    pub service_endpoint_project_references: Value,
    pub service_management_reference: Option<Value>,
    #[serde(rename = "type")]
    pub kind: Value,
    pub url: Value,
}
