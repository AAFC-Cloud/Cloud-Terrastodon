use cloud_terrastodon_azure_types::OptionalNonEmptyStringProxy;
use facet_json::RawJson;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsServiceEndpointOperationStatus {
    pub error_code: Option<RawJson<'static>>,
    #[facet(proxy = OptionalNonEmptyStringProxy)]
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

impl std::fmt::Display for AzureDevOpsServiceEndpointOperationStatusSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AzureDevOpsServiceEndpointOperationStatusSeverity::Warning => f.write_str("Warning"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[repr(C)]
pub enum AzureDevOpsServiceEndpointOperationStatusState {
    Failed,
    Ready,
}

#[cfg(test)]
mod test {
    use crate::AzureDevOpsServiceEndpointOperationStatus;
    use crate::AzureDevOpsServiceEndpointOperationStatusState;

    #[test]
    fn empty_severity_deserializes_as_none() -> eyre::Result<()> {
        let json = r#"{"severity":"","state":"Ready","statusMessage":""}"#;

        let status: AzureDevOpsServiceEndpointOperationStatus = facet_json::from_str(json)?;

        assert_eq!(status.severity, None);
        assert_eq!(
            status.state,
            AzureDevOpsServiceEndpointOperationStatusState::Ready
        );
        Ok(())
    }
}
