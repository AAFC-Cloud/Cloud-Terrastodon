use crate::prelude::ResourceGroupId;
use crate::prelude::RouteTableId;
use crate::prelude::RouteTableProperties;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct RouteTable {
    pub id: RouteTableId,
    pub name: String, // This is the name from Azure, distinct from RouteTableName in ID
    pub location: String,
    pub tags: HashMap<String, String>,
    pub properties: RouteTableProperties,
}

impl RouteTable {
    // Helper to get the ResourceGroupId from the RouteTableId
    pub fn resource_group_id(&self) -> &ResourceGroupId {
        &self.id.resource_group_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::ResourceGroupName;
    use crate::prelude::RouteTableName;
    use crate::prelude::SubscriptionId;
    use crate::scopes::Scope;
    use crate::slug::Slug;
    use eyre::Result;
    use std::str::FromStr;
    use uuid::Uuid;

    #[test]
    fn deserializes_route_table() -> Result<()> {
        let sub_id = SubscriptionId::new(Uuid::new_v4());
        let rg_name = ResourceGroupName::try_new("test-rg")?;
        let rg_id = ResourceGroupId::new(sub_id.clone(), rg_name.clone());
        let rt_name_slug = RouteTableName::try_new("test-route-table")?;
        let rt_id = RouteTableId::new(rg_id, rt_name_slug.clone());
        let json_data = serde_json::json!({
            "id": rt_id.expanded_form(),
            "name": "test-route-table-azure-name",
            "location": "eastus",
            "resource_group_name": rg_name,
            "subscription_id": sub_id.as_hyphenated().to_string(),
            "tags": {
                "environment": "test"
            },
            "properties": {
                "routes": [],
                "subnets": [],
                "resourceGuid": "12345678-1234-1234-1234-123456789012",
                "provisioningState": "Succeeded",
                "disableBgpRoutePropagation": false
            }
        });

        let deserialized_rt: RouteTable = serde_json::from_value(json_data)?;
        assert_eq!(deserialized_rt.id, rt_id);
        assert_eq!(deserialized_rt.name, "test-route-table-azure-name");
        assert_eq!(deserialized_rt.location, "eastus");
        assert_eq!(deserialized_rt.tags.get("environment").unwrap(), "test");
        assert!(deserialized_rt.properties.routes.is_empty());
        assert!(deserialized_rt.properties.subnets.is_empty());
        assert_eq!(
            deserialized_rt.properties.resource_guid,
            "12345678-1234-1234-1234-123456789012"
        );
        assert_eq!(deserialized_rt.properties.provisioning_state, "Succeeded");
        assert_eq!(
            deserialized_rt.properties.disable_bgp_route_propagation,
            Some(false)
        );

        Ok(())
    }
}
