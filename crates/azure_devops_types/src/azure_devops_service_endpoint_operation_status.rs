use cloud_terrastodon_azure_types::serde_helpers::deserialize_none_if_empty_string;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AzureDevOpsServiceEndpointOperationStatus {
    pub error_code: Option<Value>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_none_if_empty_string")]
    pub severity: Option<AzureDevOpsServiceEndpointOperationStatusSeverity>,
    pub state: AzureDevOpsServiceEndpointOperationStatusState,
    pub status_message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum AzureDevOpsServiceEndpointOperationStatusSeverity {
    Warning,
}
impl FromStr for AzureDevOpsServiceEndpointOperationStatusSeverity {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(Value::String(s.to_string())).map_err(|e| {
            eyre::eyre!(
                "Failed to parse AzureDevOpsServiceEndpointOperationStatusSeverity: {}",
                e
            )
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum AzureDevOpsServiceEndpointOperationStatusState {
    Failed,
    Ready,
}
