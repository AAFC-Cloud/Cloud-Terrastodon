use cloud_terrastodon_azure_types::SubscriptionId;
use cloud_terrastodon_azure_types::SubscriptionName;
use compact_str::CompactString;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct ServiceEndpointAzureRMData {
    pub app_object_id: Option<Uuid>,
    pub azure_spn_permissions: Option<String>,
    pub azure_spn_role_assignment_id: Option<String>,
    pub creation_mode: Option<AzureDevOpsServiceEndpointAzureRMDataCreationMode>,
    pub environment: AzureDevOpsServiceEndpointAzureRMDataEnvironment,
    pub identity_type: AzureDevOpsServiceEndpointAzureRMDataIdentityType,
    pub is_draft: Option<String>,
    pub management_group_id: Option<String>,
    pub management_group_name: Option<String>,
    pub ml_workspace_location: Option<String>,
    pub ml_workspace_name: Option<String>,
    pub obo_authorization: Option<String>,
    pub resource_group_name: Option<String>,
    pub resource_id: Option<String>,
    pub revert_scheme_deadline: Option<String>,
    pub spn_object_id: Option<Uuid>,
    pub scope_level: AzureDevOpsServiceEndpointAzureRMDataScopeLevel,
    pub subscription_id: Option<SubscriptionId>,
    pub subscription_name: Option<SubscriptionName>,
}

#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[repr(C)]
pub enum AzureDevOpsServiceEndpointAzureRMDataIdentityType {
    AppRegistrationManual,
    AppRegistrationAutomatic,
    ManagedIdentity,
    #[facet(other)]
    Other(CompactString),
}
#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[repr(C)]
pub enum AzureDevOpsServiceEndpointAzureRMDataCreationMode {
    Manual,
    Automatic,
}
#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[repr(C)]
pub enum AzureDevOpsServiceEndpointAzureRMDataScopeLevel {
    Subscription,
    ManagementGroup,
}
#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[repr(C)]
pub enum AzureDevOpsServiceEndpointAzureRMDataEnvironment {
    AzureCloud,
}
