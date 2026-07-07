use crate::AzureDevOpsServiceEndpointAuthorization;
use crate::AzureDevOpsServiceEndpointCreatedBy;
use crate::AzureDevOpsServiceEndpointData;
use crate::AzureDevOpsServiceEndpointId;
use crate::AzureDevOpsServiceEndpointKind;
use crate::AzureDevOpsServiceEndpointName;
use crate::AzureDevOpsServiceEndpointOperationStatus;
use crate::AzureDevOpsServiceEndpointOwner;
use crate::AzureDevOpsServiceEndpointProjectReference;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::ArbitraryJson;

#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsServiceEndpoint {
    pub administrators_group: Option<ArbitraryJson>,
    pub authorization: AzureDevOpsServiceEndpointAuthorization,
    pub created_by: AzureDevOpsServiceEndpointCreatedBy,
    pub data: AzureDevOpsServiceEndpointData,
    pub description: String,
    pub group_scope_id: Option<ArbitraryJson>,
    pub id: AzureDevOpsServiceEndpointId,
    pub is_outdated: bool,
    pub is_ready: bool,
    pub is_shared: bool,
    pub name: AzureDevOpsServiceEndpointName,
    pub operation_status: Option<AzureDevOpsServiceEndpointOperationStatus>,
    pub owner: AzureDevOpsServiceEndpointOwner,
    pub readers_group: Option<ArbitraryJson>,
    pub service_endpoint_project_references: Vec<AzureDevOpsServiceEndpointProjectReference>,
    pub service_management_reference: Option<ArbitraryJson>,
    #[facet(rename = "type")]
    pub kind: AzureDevOpsServiceEndpointKind,
    pub url: ArbitraryJson,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsServiceEndpoint);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsServiceEndpoint);
cloud_terrastodon_registry::register_arbitrary!(Vec<AzureDevOpsServiceEndpoint>);
