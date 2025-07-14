use compact_str::CompactString;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AzureDevOpsServiceEndpointKind {
    AWS,
    AWSServiceEndpoint,
    AzureRM,
    DockerRegistry,
    ExternalTFS,
    SSH,
    Other(CompactString),
}

impl Serialize for AzureDevOpsServiceEndpointKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match self {
            AzureDevOpsServiceEndpointKind::AWS => "aws",
            AzureDevOpsServiceEndpointKind::AWSServiceEndpoint => "awsserviceendpoint",
            AzureDevOpsServiceEndpointKind::AzureRM => "azurerm",
            AzureDevOpsServiceEndpointKind::DockerRegistry => "dockerregistry",
            AzureDevOpsServiceEndpointKind::ExternalTFS => "externaltfs",
            AzureDevOpsServiceEndpointKind::SSH => "ssh",
            AzureDevOpsServiceEndpointKind::Other(val) => val.as_str(),
        };
        serializer.serialize_str(s)
    }
}

impl<'de> Deserialize<'de> for AzureDevOpsServiceEndpointKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = AzureDevOpsServiceEndpointKind;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string representing a service endpoint kind")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v.to_ascii_lowercase().as_str() {
                    "aws" => Ok(AzureDevOpsServiceEndpointKind::AWS),
                    "awsserviceendpoint" => Ok(AzureDevOpsServiceEndpointKind::AWSServiceEndpoint),
                    "azurerm" => Ok(AzureDevOpsServiceEndpointKind::AzureRM),
                    "dockerregistry" => Ok(AzureDevOpsServiceEndpointKind::DockerRegistry),
                    "externaltfs" => Ok(AzureDevOpsServiceEndpointKind::ExternalTFS),
                    "ssh" => Ok(AzureDevOpsServiceEndpointKind::SSH),
                    other => Ok(AzureDevOpsServiceEndpointKind::Other(CompactString::from(
                        other,
                    ))),
                }
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use compact_str::CompactString;
    use serde_json;

    #[test]
    fn test_serialize_known_variants() {
        assert_eq!(
            serde_json::to_string(&AzureDevOpsServiceEndpointKind::AWS).unwrap(),
            "\"aws\""
        );
        assert_eq!(
            serde_json::to_string(&AzureDevOpsServiceEndpointKind::AWSServiceEndpoint).unwrap(),
            "\"awsserviceendpoint\""
        );
        assert_eq!(
            serde_json::to_string(&AzureDevOpsServiceEndpointKind::AzureRM).unwrap(),
            "\"azurerm\""
        );
        assert_eq!(
            serde_json::to_string(&AzureDevOpsServiceEndpointKind::DockerRegistry).unwrap(),
            "\"dockerregistry\""
        );
        assert_eq!(
            serde_json::to_string(&AzureDevOpsServiceEndpointKind::ExternalTFS).unwrap(),
            "\"externaltfs\""
        );
        assert_eq!(
            serde_json::to_string(&AzureDevOpsServiceEndpointKind::SSH).unwrap(),
            "\"ssh\""
        );
    }

    #[test]
    fn test_serialize_other_variant() {
        let other = AzureDevOpsServiceEndpointKind::Other(CompactString::from("custom"));
        assert_eq!(serde_json::to_string(&other).unwrap(), "\"custom\"");
    }

    #[test]
    fn test_deserialize_known_variants_case_insensitive() {
        assert_eq!(
            serde_json::from_str::<AzureDevOpsServiceEndpointKind>("\"AWS\"").unwrap(),
            AzureDevOpsServiceEndpointKind::AWS
        );
        assert_eq!(
            serde_json::from_str::<AzureDevOpsServiceEndpointKind>("\"awsserviceendpoint\"")
                .unwrap(),
            AzureDevOpsServiceEndpointKind::AWSServiceEndpoint
        );
        assert_eq!(
            serde_json::from_str::<AzureDevOpsServiceEndpointKind>("\"AZURERM\"").unwrap(),
            AzureDevOpsServiceEndpointKind::AzureRM
        );
        assert_eq!(
            serde_json::from_str::<AzureDevOpsServiceEndpointKind>("\"DockerRegistry\"").unwrap(),
            AzureDevOpsServiceEndpointKind::DockerRegistry
        );
        assert_eq!(
            serde_json::from_str::<AzureDevOpsServiceEndpointKind>("\"externaltfs\"").unwrap(),
            AzureDevOpsServiceEndpointKind::ExternalTFS
        );
        assert_eq!(
            serde_json::from_str::<AzureDevOpsServiceEndpointKind>("\"ssh\"").unwrap(),
            AzureDevOpsServiceEndpointKind::SSH
        );
    }

    #[test]
    fn test_deserialize_other_variant() {
        let val = "\"somethingelse\"";
        let parsed: AzureDevOpsServiceEndpointKind = serde_json::from_str(val).unwrap();
        assert_eq!(
            parsed,
            AzureDevOpsServiceEndpointKind::Other(CompactString::from("somethingelse"))
        );
    }
}
