use crate::scopes::ScopeImpl;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::Visitor;
use serde::de::{self};
use std::fmt;

// TODO: this should be converted to an enum to prevent states where `kind` doesn't match `id`
//#[serde(tag = "type")]
#[derive(Debug, Serialize, Deserialize)]
pub struct EligibleChildResource {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: EligibleChildResourceKind,
    pub id: ScopeImpl,
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
            EligibleChildResourceKind::Other(s) => serializer.serialize_str(s),
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
    use crate::management_groups::ManagementGroupId;
    use crate::prelude::TestResourceId;
    use crate::scopes::Scope;
    use serde_json;

    #[test]
    fn test_serialization() {
        let thing_id = ManagementGroupId::from_name("test-mg");
        let resource = EligibleChildResource {
            name: String::from("Example Resource"),
            kind: EligibleChildResourceKind::ManagementGroup,
            id: thing_id.as_scope_impl(),
        };

        let json = serde_json::to_string(&resource).unwrap();
        let expected_json = format!(
            r#"{{"name":"Example Resource","type":"managementgroup","id":"{}"}}"#,
            thing_id.expanded_form()
        );

        assert_eq!(json, expected_json);
    }

    #[test]
    fn test_deserialization() {
        let thing_id = ManagementGroupId::from_name("test-mg");
        let json = format!(
            r#"{{"name":"Example Resource","type":"managementgroup","id":"{}"}}"#,
            thing_id.expanded_form()
        );
        let resource: EligibleChildResource = serde_json::from_str(&json).unwrap();

        assert_eq!(resource.name, "Example Resource");
        assert_eq!(resource.id, thing_id.as_scope_impl());
        match resource.kind {
            EligibleChildResourceKind::ManagementGroup => {}
            _ => panic!("Expected ManagementGroup"),
        }
    }

    #[test]
    fn test_serialization_with_other() {
        let thing_id = TestResourceId::new("resource-id-456");
        let resource = EligibleChildResource {
            name: String::from("Another Resource"),
            kind: EligibleChildResourceKind::Other(String::from("customtype")),
            id: thing_id.as_scope_impl(),
        };

        let json = serde_json::to_string(&resource).unwrap();
        let expected_json = format!(
            r#"{{"name":"Another Resource","type":"customtype","id":"{}"}}"#,
            thing_id.expanded_form()
        );

        assert_eq!(json, expected_json);
    }

    #[test]
    fn test_deserialization_with_other() {
        let thing_id = TestResourceId::new("test-resource-123");
        let json = format!(
            r#"{{"name":"Another Resource","type":"customtype","id":"{}"}}"#,
            thing_id.expanded_form()
        );
        let resource: EligibleChildResource = serde_json::from_str(&json).unwrap();

        assert_eq!(resource.name, "Another Resource");
        assert_eq!(resource.id, thing_id.as_scope_impl());
        match resource.kind {
            EligibleChildResourceKind::Other(ref s) if s == "customtype" => {}
            _ => panic!("Expected Other with value 'customtype'"),
        }
    }
}
