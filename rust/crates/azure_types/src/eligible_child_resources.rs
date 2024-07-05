use serde::de::Visitor;
use serde::de::{self};
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
pub struct EligibleChildResource {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: EligibleChildResourceKind,
    pub id: String,
}
impl std::fmt::Display for EligibleChildResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:?} - {}", self.kind, self.name))
    }
}

#[derive(Debug)]
pub enum EligibleChildResourceKind {
    ManagementGroup,
    Subscription,
    ResourceGroup,
    Other(String),
}

impl Serialize for EligibleChildResourceKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            EligibleChildResourceKind::ManagementGroup => {
                serializer.serialize_str("managementgroup")
            }
            EligibleChildResourceKind::Subscription => serializer.serialize_str("subscription"),
            EligibleChildResourceKind::ResourceGroup => serializer.serialize_str("resourcegroup"),
            EligibleChildResourceKind::Other(ref s) => serializer.serialize_str(s),
        }
    }
}

struct EligibleChildResourceKindVisitor;

impl<'de> Visitor<'de> for EligibleChildResourceKindVisitor {
    type Value = EligibleChildResourceKind;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string representing the resource kind")
    }

    fn visit_str<E>(self, value: &str) -> Result<EligibleChildResourceKind, E>
    where
        E: de::Error,
    {
        Ok(match value {
            "managementgroup" => EligibleChildResourceKind::ManagementGroup,
            "subscription" => EligibleChildResourceKind::Subscription,
            "resourcegroup" => EligibleChildResourceKind::ResourceGroup,
            other => EligibleChildResourceKind::Other(other.to_string()),
        })
    }
}

impl<'de> Deserialize<'de> for EligibleChildResourceKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(EligibleChildResourceKindVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_serialization() {
        let resource = EligibleChildResource {
            name: String::from("Example Resource"),
            kind: EligibleChildResourceKind::ManagementGroup,
            id: String::from("resource-id-123"),
        };

        let json = serde_json::to_string(&resource).unwrap();
        let expected_json =
            r#"{"name":"Example Resource","type":"managementgroup","id":"resource-id-123"}"#;

        assert_eq!(json, expected_json);
    }

    #[test]
    fn test_deserialization() {
        let json = r#"{"name":"Example Resource","type":"managementgroup","id":"resource-id-123"}"#;
        let resource: EligibleChildResource = serde_json::from_str(json).unwrap();

        assert_eq!(resource.name, "Example Resource");
        assert_eq!(resource.id, "resource-id-123");
        match resource.kind {
            EligibleChildResourceKind::ManagementGroup => {}
            _ => panic!("Expected ManagementGroup"),
        }
    }

    #[test]
    fn test_serialization_with_other() {
        let resource = EligibleChildResource {
            name: String::from("Another Resource"),
            kind: EligibleChildResourceKind::Other(String::from("customtype")),
            id: String::from("resource-id-456"),
        };

        let json = serde_json::to_string(&resource).unwrap();
        let expected_json =
            r#"{"name":"Another Resource","type":"customtype","id":"resource-id-456"}"#;

        assert_eq!(json, expected_json);
    }

    #[test]
    fn test_deserialization_with_other() {
        let json = r#"{"name":"Another Resource","type":"customtype","id":"resource-id-456"}"#;
        let resource: EligibleChildResource = serde_json::from_str(json).unwrap();

        assert_eq!(resource.name, "Another Resource");
        assert_eq!(resource.id, "resource-id-456");
        match resource.kind {
            EligibleChildResourceKind::Other(ref s) if s == "customtype" => {}
            _ => panic!("Expected Other with value 'customtype'"),
        }
    }
}
