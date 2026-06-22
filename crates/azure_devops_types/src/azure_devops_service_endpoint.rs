use crate::AzureDevOpsServiceEndpointAuthorization;
use crate::AzureDevOpsServiceEndpointCreatedBy;
use crate::AzureDevOpsServiceEndpointData;
use crate::AzureDevOpsServiceEndpointId;
use crate::AzureDevOpsServiceEndpointKind;
use crate::AzureDevOpsServiceEndpointName;
use crate::AzureDevOpsServiceEndpointOperationStatus;
use crate::AzureDevOpsServiceEndpointOwner;
use crate::AzureDevOpsServiceEndpointProjectReference;
use facet_json::RawJson;

#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsServiceEndpoint {
    pub administrators_group: Option<RawJson<'static>>,
    pub authorization: AzureDevOpsServiceEndpointAuthorization,
    pub created_by: AzureDevOpsServiceEndpointCreatedBy,
    pub data: AzureDevOpsServiceEndpointData,
    pub description: String,
    pub group_scope_id: Option<RawJson<'static>>,
    pub id: AzureDevOpsServiceEndpointId,
    pub is_outdated: bool,
    pub is_ready: bool,
    pub is_shared: bool,
    pub name: AzureDevOpsServiceEndpointName,
    pub operation_status: Option<AzureDevOpsServiceEndpointOperationStatus>,
    pub owner: AzureDevOpsServiceEndpointOwner,
    pub readers_group: Option<RawJson<'static>>,
    pub service_endpoint_project_references: Vec<AzureDevOpsServiceEndpointProjectReference>,
    pub service_management_reference: Option<RawJson<'static>>,
    #[facet(rename = "type")]
    pub kind: AzureDevOpsServiceEndpointKind,
    pub url: RawJson<'static>,
}
