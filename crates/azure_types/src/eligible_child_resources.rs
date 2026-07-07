use crate::scopes::ScopeImpl;
use arbitrary::Arbitrary;
use std::fmt;

// TODO: this should be converted to an enum to prevent states where `kind` doesn't match `id`
#[derive(Debug, Arbitrary, facet::Facet, PartialEq)]
pub struct EligibleChildResource {
    pub name: String,
    #[facet(rename = "type")]
    pub kind: EligibleChildResourceKind,
    pub id: ScopeImpl,
}
impl std::fmt::Display for EligibleChildResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:?} - {}", self.kind, self.name))
    }
}

#[derive(Debug, PartialEq, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum EligibleChildResourceKind {
    ManagementGroup,
    Subscription,
    ResourceGroup,
    Other(String),
}
crate::impl_facet_string_proxy!(EligibleChildResourceKind, value => value.to_string());

impl std::fmt::Display for EligibleChildResourceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EligibleChildResourceKind::ManagementGroup => f.write_str("managementgroup"),
            EligibleChildResourceKind::Subscription => f.write_str("subscription"),
            EligibleChildResourceKind::ResourceGroup => f.write_str("resourcegroup"),
            EligibleChildResourceKind::Other(s) => f.write_str(s),
        }
    }
}

impl std::str::FromStr for EligibleChildResourceKind {
    type Err = std::convert::Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            "managementgroup" => EligibleChildResourceKind::ManagementGroup,
            "subscription" => EligibleChildResourceKind::Subscription,
            "resourcegroup" => EligibleChildResourceKind::ResourceGroup,
            other => EligibleChildResourceKind::Other(other.to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TestResourceId;
    use crate::management_groups::ManagementGroupId;
    use crate::scopes::Scope;

    #[test]
    fn test_serialization() -> eyre::Result<()> {
        let thing_id = ManagementGroupId::from_name("test-mg");
        let resource = EligibleChildResource {
            name: String::from("Example Resource"),
            kind: EligibleChildResourceKind::ManagementGroup,
            id: thing_id.as_scope_impl(),
        };

        let json = facet_json::to_string(&resource)?;
        let expected_json = format!(
            r#"{{"name":"Example Resource","type":"managementgroup","id":"{}"}}"#,
            thing_id.expanded_form()
        );

        assert_eq!(json, expected_json);
        let reparsed = facet_json::from_str::<EligibleChildResource>(&json)?;
        assert_eq!(resource, reparsed);
        Ok(())
    }

    #[test]
    fn test_deserialization() -> eyre::Result<()> {
        let thing_id = ManagementGroupId::from_name("test-mg");
        let json = format!(
            r#"{{"name":"Example Resource","type":"managementgroup","id":"{}"}}"#,
            thing_id.expanded_form()
        );
        let resource: EligibleChildResource = facet_json::from_str(&json)?;

        assert_eq!(resource.name, "Example Resource");
        assert_eq!(resource.id, thing_id.as_scope_impl());
        match resource.kind {
            EligibleChildResourceKind::ManagementGroup => {}
            _ => panic!("Expected ManagementGroup"),
        }
        Ok(())
    }

    #[test]
    fn test_serialization_with_other() -> eyre::Result<()> {
        let thing_id = TestResourceId::new("resource-id-456");
        let resource = EligibleChildResource {
            name: String::from("Another Resource"),
            kind: EligibleChildResourceKind::Other(String::from("customtype")),
            id: thing_id.as_scope_impl(),
        };

        let json = facet_json::to_string(&resource)?;
        let expected_json = format!(
            r#"{{"name":"Another Resource","type":"customtype","id":"{}"}}"#,
            thing_id.expanded_form()
        );

        assert_eq!(json, expected_json);
        let reparsed = facet_json::from_str::<EligibleChildResource>(&json)?;
        assert_eq!(resource, reparsed);
        Ok(())
    }

    #[test]
    fn test_deserialization_with_other() -> eyre::Result<()> {
        let thing_id = TestResourceId::new("test-resource-123");
        let json = format!(
            r#"{{"name":"Another Resource","type":"customtype","id":"{}"}}"#,
            thing_id.expanded_form()
        );
        let resource: EligibleChildResource = facet_json::from_str(&json)?;

        assert_eq!(resource.name, "Another Resource");
        assert_eq!(resource.id, thing_id.as_scope_impl());
        match resource.kind {
            EligibleChildResourceKind::Other(ref s) if s == "customtype" => {}
            _ => panic!("Expected Other with value 'customtype'"),
        }
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(EligibleChildResourceKind);
cloud_terrastodon_registry::register_arbitrary!(EligibleChildResourceKind);
cloud_terrastodon_registry::register_thing!(EligibleChildResource);
cloud_terrastodon_registry::register_arbitrary!(EligibleChildResource);
cloud_terrastodon_registry::register_arbitrary!(Vec<EligibleChildResource>);
