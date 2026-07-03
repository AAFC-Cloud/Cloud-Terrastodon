use compact_str::CompactString;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[facet(opaque, proxy = String)]
#[repr(C)]
pub enum AzureDevOpsServiceEndpointKind {
    AWS,
    AWSServiceEndpoint,
    AzureRM,
    DockerRegistry,
    ExternalTFS,
    SSH,
    Other(CompactString),
}

impl From<&AzureDevOpsServiceEndpointKind> for String {
    fn from(value: &AzureDevOpsServiceEndpointKind) -> Self {
        match value {
            AzureDevOpsServiceEndpointKind::AWS => "aws",
            AzureDevOpsServiceEndpointKind::AWSServiceEndpoint => "awsserviceendpoint",
            AzureDevOpsServiceEndpointKind::AzureRM => "azurerm",
            AzureDevOpsServiceEndpointKind::DockerRegistry => "dockerregistry",
            AzureDevOpsServiceEndpointKind::ExternalTFS => "externaltfs",
            AzureDevOpsServiceEndpointKind::SSH => "ssh",
            AzureDevOpsServiceEndpointKind::Other(val) => val.as_str(),
        }
        .to_owned()
    }
}

impl TryFrom<String> for AzureDevOpsServiceEndpointKind {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl FromStr for AzureDevOpsServiceEndpointKind {
    type Err = eyre::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value.to_ascii_lowercase().as_str() {
            "aws" => AzureDevOpsServiceEndpointKind::AWS,
            "awsserviceendpoint" => AzureDevOpsServiceEndpointKind::AWSServiceEndpoint,
            "azurerm" => AzureDevOpsServiceEndpointKind::AzureRM,
            "dockerregistry" => AzureDevOpsServiceEndpointKind::DockerRegistry,
            "externaltfs" => AzureDevOpsServiceEndpointKind::ExternalTFS,
            "ssh" => AzureDevOpsServiceEndpointKind::SSH,
            other => AzureDevOpsServiceEndpointKind::Other(CompactString::from(other)),
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsServiceEndpointKind);

#[cfg(test)]
mod tests {
    use super::*;
    use compact_str::CompactString;

    #[test]
    fn test_serialize_known_variants() {
        assert_eq!(
            facet_json::to_string(&AzureDevOpsServiceEndpointKind::AWS).unwrap(),
            "\"aws\""
        );
        assert_eq!(
            facet_json::to_string(&AzureDevOpsServiceEndpointKind::AWSServiceEndpoint).unwrap(),
            "\"awsserviceendpoint\""
        );
        assert_eq!(
            facet_json::to_string(&AzureDevOpsServiceEndpointKind::AzureRM).unwrap(),
            "\"azurerm\""
        );
        assert_eq!(
            facet_json::to_string(&AzureDevOpsServiceEndpointKind::DockerRegistry).unwrap(),
            "\"dockerregistry\""
        );
        assert_eq!(
            facet_json::to_string(&AzureDevOpsServiceEndpointKind::ExternalTFS).unwrap(),
            "\"externaltfs\""
        );
        assert_eq!(
            facet_json::to_string(&AzureDevOpsServiceEndpointKind::SSH).unwrap(),
            "\"ssh\""
        );
    }

    #[test]
    fn test_serialize_other_variant() {
        let other = AzureDevOpsServiceEndpointKind::Other(CompactString::from("custom"));
        assert_eq!(facet_json::to_string(&other).unwrap(), "\"custom\"");
    }

    #[test]
    fn test_deserialize_known_variants_case_insensitive() {
        assert_eq!(
            facet_json::from_str::<AzureDevOpsServiceEndpointKind>("\"AWS\"").unwrap(),
            AzureDevOpsServiceEndpointKind::AWS
        );
        assert_eq!(
            facet_json::from_str::<AzureDevOpsServiceEndpointKind>("\"awsserviceendpoint\"")
                .unwrap(),
            AzureDevOpsServiceEndpointKind::AWSServiceEndpoint
        );
        assert_eq!(
            facet_json::from_str::<AzureDevOpsServiceEndpointKind>("\"AZURERM\"").unwrap(),
            AzureDevOpsServiceEndpointKind::AzureRM
        );
        assert_eq!(
            facet_json::from_str::<AzureDevOpsServiceEndpointKind>("\"DockerRegistry\"").unwrap(),
            AzureDevOpsServiceEndpointKind::DockerRegistry
        );
        assert_eq!(
            facet_json::from_str::<AzureDevOpsServiceEndpointKind>("\"externaltfs\"").unwrap(),
            AzureDevOpsServiceEndpointKind::ExternalTFS
        );
        assert_eq!(
            facet_json::from_str::<AzureDevOpsServiceEndpointKind>("\"ssh\"").unwrap(),
            AzureDevOpsServiceEndpointKind::SSH
        );
    }

    #[test]
    fn test_deserialize_other_variant() {
        let val = "\"somethingelse\"";
        let parsed: AzureDevOpsServiceEndpointKind = facet_json::from_str(val).unwrap();
        assert_eq!(
            parsed,
            AzureDevOpsServiceEndpointKind::Other(CompactString::from("somethingelse"))
        );
    }
}

