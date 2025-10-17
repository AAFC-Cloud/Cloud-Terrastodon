use cloud_terrastodon_azure_types::prelude::SubscriptionId;
use cloud_terrastodon_azure_types::prelude::SubscriptionName;
use cloud_terrastodon_azure_types::serde_helpers::deserialize_none_if_empty_string;
use compact_str::CompactString;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct ServiceEndpointAzureRMData {
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_none_if_empty_string")]
    pub app_object_id: Option<Uuid>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_none_if_empty_string")]
    pub azure_spn_permissions: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_none_if_empty_string")]
    pub azure_spn_role_assignment_id: Option<String>,
    pub creation_mode: Option<AzureDevOpsServiceEndpointAzureRMDataCreationMode>,
    pub environment: AzureDevOpsServiceEndpointAzureRMDataEnvironment,
    pub identity_type: AzureDevOpsServiceEndpointAzureRMDataIdentityType,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_none_if_empty_string")]
    pub is_draft: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_none_if_empty_string")]
    pub management_group_id: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_none_if_empty_string")]
    pub management_group_name: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_none_if_empty_string")]
    pub ml_workspace_location: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_none_if_empty_string")]
    pub ml_workspace_name: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_none_if_empty_string")]
    pub obo_authorization: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_none_if_empty_string")]
    pub resource_group_name: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_none_if_empty_string")]
    pub resource_id: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_none_if_empty_string")]
    pub revert_scheme_deadline: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_none_if_empty_string")]
    pub spn_object_id: Option<Uuid>,
    pub scope_level: AzureDevOpsServiceEndpointAzureRMDataScopeLevel,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_none_if_empty_string")]
    pub subscription_id: Option<SubscriptionId>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_none_if_empty_string")]
    pub subscription_name: Option<SubscriptionName>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum AzureDevOpsServiceEndpointAzureRMDataIdentityType {
    AppRegistrationManual,
    AppRegistrationAutomatic,
    ManagedIdentity,
    #[serde(untagged)]
    Other(CompactString),
}
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum AzureDevOpsServiceEndpointAzureRMDataCreationMode {
    Manual,
    Automatic,
}
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum AzureDevOpsServiceEndpointAzureRMDataScopeLevel {
    Subscription,
    ManagementGroup,
}
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum AzureDevOpsServiceEndpointAzureRMDataEnvironment {
    AzureCloud,
}
