use crate::prelude::AzureDevOpsServiceEndpointAuthorization;
use crate::prelude::AzureDevOpsServiceEndpointCreatedBy;
use crate::prelude::AzureDevOpsServiceEndpointData;
use crate::prelude::AzureDevOpsServiceEndpointId;
use crate::prelude::AzureDevOpsServiceEndpointKind;
use crate::prelude::AzureDevOpsServiceEndpointName;
use crate::prelude::AzureDevOpsServiceEndpointOperationStatus;
use crate::prelude::AzureDevOpsServiceEndpointOwner;
use crate::prelude::AzureDevOpsServiceEndpointProjectReference;
use crate::prelude::ServiceEndpointAzureRMData;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct AzureDevOpsServiceEndpoint {
    pub administrators_group: Option<Value>,
    pub authorization: AzureDevOpsServiceEndpointAuthorization,
    pub created_by: AzureDevOpsServiceEndpointCreatedBy,
    pub data: AzureDevOpsServiceEndpointData,
    pub description: String,
    pub group_scope_id: Option<Value>,
    pub id: AzureDevOpsServiceEndpointId,
    pub is_outdated: bool,
    pub is_ready: bool,
    pub is_shared: bool,
    pub name: AzureDevOpsServiceEndpointName,
    pub operation_status: Option<AzureDevOpsServiceEndpointOperationStatus>,
    pub owner: AzureDevOpsServiceEndpointOwner,
    pub readers_group: Option<Value>,
    pub service_endpoint_project_references: Vec<AzureDevOpsServiceEndpointProjectReference>,
    pub service_management_reference: Option<Value>,
    #[serde(rename = "type")]
    pub kind: AzureDevOpsServiceEndpointKind,
    pub url: Value,
}
// 1)  A mirror struct with `data` still untyped.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawEndpoint {
    administrators_group: Option<Value>,
    authorization: AzureDevOpsServiceEndpointAuthorization,
    created_by: AzureDevOpsServiceEndpointCreatedBy,
    data: Value, // <-- untyped!
    description: String,
    group_scope_id: Option<Value>,
    id: AzureDevOpsServiceEndpointId,
    is_outdated: bool,
    is_ready: bool,
    is_shared: bool,
    name: AzureDevOpsServiceEndpointName,
    operation_status: Option<AzureDevOpsServiceEndpointOperationStatus>,
    owner: AzureDevOpsServiceEndpointOwner,
    readers_group: Option<Value>,
    service_endpoint_project_references: Vec<AzureDevOpsServiceEndpointProjectReference>,
    service_management_reference: Option<Value>,
    #[serde(rename = "type")]
    kind: AzureDevOpsServiceEndpointKind,
    url: Value,
}

impl<'de> Deserialize<'de> for AzureDevOpsServiceEndpoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // 2)  Deserialize into RawEndpoint first.
        let raw = RawEndpoint::deserialize(deserializer)?;

        // 3)  Re-deserialize `raw.data` based on `raw.kind`.
        let data = match raw.kind {
            AzureDevOpsServiceEndpointKind::AzureRM => {
                let typed: ServiceEndpointAzureRMData =
                    serde_json::from_value(raw.data).map_err(serde::de::Error::custom)?;
                AzureDevOpsServiceEndpointData::AzureRM(typed)
            }
            // add more arms when you introduce more strongly-typed kinds
            _ => AzureDevOpsServiceEndpointData::Other(raw.data),
        };

        // 4)  Build the final value.
        Ok(AzureDevOpsServiceEndpoint {
            administrators_group: raw.administrators_group,
            authorization: raw.authorization,
            created_by: raw.created_by,
            data,
            description: raw.description,
            group_scope_id: raw.group_scope_id,
            id: raw.id,
            is_outdated: raw.is_outdated,
            is_ready: raw.is_ready,
            is_shared: raw.is_shared,
            name: raw.name,
            operation_status: raw.operation_status,
            owner: raw.owner,
            readers_group: raw.readers_group,
            service_endpoint_project_references: raw.service_endpoint_project_references,
            service_management_reference: raw.service_management_reference,
            kind: raw.kind,
            url: raw.url,
        })
    }
}
