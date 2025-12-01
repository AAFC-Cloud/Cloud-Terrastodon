use crate::prelude::ServiceGroupId;
use crate::prelude::ServiceGroupName;
use crate::scopes::AsScope;
use crate::scopes::Scope;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct ServiceGroupProperties {
    #[serde(rename = "provisioningState")]
    pub provisioning_state: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub parent: Option<ServiceGroupParent>,
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct ServiceGroupParent {
    #[serde(rename = "resourceId")]
    pub resource_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn properties_round_trip() {
        let json_value = json!({
            "provisioningState": "Succeeded",
            "displayName": "MyServiceGroup",
            "parent": {"resourceId": "/providers/Microsoft.Management/serviceGroups/parent"},
            "custom": 42
        });
        let props: ServiceGroupProperties = serde_json::from_value(json_value.clone()).unwrap();
        assert_eq!(props.provisioning_state.as_deref(), Some("Succeeded"));
        assert_eq!(props.display_name.as_deref(), Some("MyServiceGroup"));
        assert_eq!(
            props.parent.as_ref().unwrap().resource_id.as_deref(),
            Some("/providers/Microsoft.Management/serviceGroups/parent")
        );
        assert!(props.additional_properties.contains_key("custom"));

        let serialized = serde_json::to_value(&props).unwrap();
        assert_eq!(serialized, json_value);
    }
}
