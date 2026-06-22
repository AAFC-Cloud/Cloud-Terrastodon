use crate::ServiceGroupId;
use crate::ServiceGroupName;
use crate::scopes::AsScope;
use crate::scopes::Scope;
use facet_json::RawJson;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone, facet::Facet)]
pub struct ServiceGroup {
    pub id: ServiceGroupId,
    pub name: ServiceGroupName,
    pub properties: ServiceGroupProperties,
}

impl AsScope for ServiceGroup {
    fn as_scope(&self) -> &impl Scope {
        &self.id
    }
}

impl AsScope for &ServiceGroup {
    fn as_scope(&self) -> &impl Scope {
        &self.id
    }
}

impl std::fmt::Display for ServiceGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, facet::Facet)]
pub struct ServiceGroupProperties {
    #[facet(rename = "provisioningState")]
    pub provisioning_state: Option<String>,
    #[facet(rename = "displayName")]
    pub display_name: Option<String>,
    pub parent: Option<ServiceGroupParent>,
    #[facet(flatten)]
    pub additional_properties: HashMap<String, RawJson<'static>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Default, facet::Facet)]
pub struct ServiceGroupParent {
    #[facet(rename = "resourceId")]
    pub resource_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn properties_round_trip() -> eyre::Result<()> {
        let json = r#"{
            "provisioningState": "Succeeded",
            "displayName": "MyServiceGroup",
            "parent": {"resourceId": "/providers/Microsoft.Management/serviceGroups/parent"},
            "custom": 42
        }"#;
        let props: ServiceGroupProperties = facet_json::from_str(json)?;
        assert_eq!(props.provisioning_state.as_deref(), Some("Succeeded"));
        assert_eq!(props.display_name.as_deref(), Some("MyServiceGroup"));
        assert_eq!(
            props.parent.as_ref().unwrap().resource_id.as_deref(),
            Some("/providers/Microsoft.Management/serviceGroups/parent")
        );
        assert!(props.additional_properties.contains_key("custom"));
        assert_eq!(props.additional_properties["custom"].as_str(), "42");

        let reparsed: ServiceGroupProperties =
            facet_json::from_str(&facet_json::to_string(&props)?)?;
        assert_eq!(props, reparsed);
        Ok(())
    }

    #[test]
    fn properties_json_round_trips() -> eyre::Result<()> {
        let json = r#"{
            "provisioningState": "Succeeded",
            "displayName": "MyServiceGroup",
            "parent": {"resourceId": "/providers/Microsoft.Management/serviceGroups/parent"},
            "custom": 42
        }"#;

        let props: ServiceGroupProperties = facet_json::from_str(json)?;
        let reparsed: ServiceGroupProperties =
            facet_json::from_str(&facet_json::to_string(&props)?)?;
        assert_eq!(props, reparsed);
        Ok(())
    }
}
