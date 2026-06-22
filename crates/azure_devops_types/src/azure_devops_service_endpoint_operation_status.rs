use facet_json::RawJson;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsServiceEndpointOperationStatus {
    pub error_code: Option<RawJson<'static>>,
    pub severity: Option<AzureDevOpsServiceEndpointOperationStatusSeverity>,
    pub state: AzureDevOpsServiceEndpointOperationStatusState,
    pub status_message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[repr(C)]
pub enum AzureDevOpsServiceEndpointOperationStatusSeverity {
    Warning,
}
impl FromStr for AzureDevOpsServiceEndpointOperationStatusSeverity {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Warning" => Ok(Self::Warning),
            value => eyre::bail!(
                "Failed to parse AzureDevOpsServiceEndpointOperationStatusSeverity: {}",
                value
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[repr(C)]
pub enum AzureDevOpsServiceEndpointOperationStatusState {
    Failed,
    Ready,
}
