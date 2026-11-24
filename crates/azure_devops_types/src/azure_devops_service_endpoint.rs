use crate::AzureDevOpsIdentityReference;
use crate::AzureDevOpsServiceEndpointAuthorization;
use crate::AzureDevOpsServiceEndpointData;
use crate::AzureDevOpsServiceEndpointId;
use crate::AzureDevOpsServiceEndpointKind;
use crate::AzureDevOpsServiceEndpointName;
use crate::AzureDevOpsServiceEndpointOperationStatus;
use crate::AzureDevOpsServiceEndpointOwner;
use crate::AzureDevOpsServiceEndpointProjectReference;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::ArbitraryJson;
use facet::Facet;

/// A service connection returned by the Azure DevOps [Service Endpoints API 7.1](https://learn.microsoft.com/en-us/rest/api/azure/devops/serviceendpoint/endpoints/get-service-endpoints?view=azure-devops-rest-7.1#serviceendpoint).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsServiceEndpoint {
    pub administrators_group: Option<ArbitraryJson>,
    pub authorization: AzureDevOpsServiceEndpointAuthorization,
    pub created_by: AzureDevOpsIdentityReference,
    pub data: AzureDevOpsServiceEndpointData,
    pub description: String,
    pub group_scope_id: Option<ArbitraryJson>,
    pub id: AzureDevOpsServiceEndpointId,
    /// Some endpoint responses omit this flag, including the documented list example.
    pub is_outdated: Option<bool>,
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
