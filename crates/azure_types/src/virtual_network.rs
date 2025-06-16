use crate::prelude::ResourceGroupId;
use crate::prelude::VirtualNetworkId;
use crate::prelude::VirtualNetworkProperties;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct VirtualNetwork {
    pub id: VirtualNetworkId,
    pub name: String, // This is the name from Azure, distinct from VirtualNetworkName in ID
    pub location: String,
    pub tags: Option<HashMap<String, String>>,
    pub properties: VirtualNetworkProperties,
}

impl VirtualNetwork {
    // Helper to get the ResourceGroupId from the VirtualNetworkId
    pub fn resource_group_id(&self) -> &ResourceGroupId {
        &self.id.resource_group_id
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::prelude::ResourceGroupName;
    use crate::prelude::SubscriptionId;
    use crate::prelude::VirtualNetworkName;
    use crate::scopes::Scope;
    use crate::slug::Slug;
    use eyre::Result;
    use ipnetwork::Ipv4Network;
    use uuid::Uuid;

    #[test]
    fn deserializes_virtual_network() -> Result<()> {
        let sub_id = SubscriptionId::new(Uuid::new_v4());
        let rg_name = ResourceGroupName::try_new("test-rg")?;
        let rg_id = ResourceGroupId::new(sub_id.clone(), rg_name.clone());
        let vnet_name_slug = VirtualNetworkName::try_new("test-vnet")?;
        let vnet_id = VirtualNetworkId::new(rg_id, vnet_name_slug.clone());
        let json_data = serde_json::json!({
            "id": vnet_id.expanded_form(),
            "name": "test-vnet-azure-name",
            "location": "eastus",
            "resource_group_name": rg_name,
            "subscription_id": sub_id.as_hyphenated().to_string(),
            "tags": {
                "environment": "test"
            },
            "properties": {
                "addressSpace": {
                    "addressPrefixes": ["10.0.0.0/16"]
                },
                "subnets": []
            }
        });

        let deserialized_vnet: VirtualNetwork = serde_json::from_value(json_data)?;
        assert_eq!(deserialized_vnet.id, vnet_id);
        assert_eq!(deserialized_vnet.name, "test-vnet-azure-name");
        assert_eq!(deserialized_vnet.location, "eastus");
        assert_eq!(
            deserialized_vnet.tags.unwrap().get("environment").unwrap(),
            "test"
        );
        assert_eq!(
            deserialized_vnet.properties.address_space.address_prefixes,
            vec![Ipv4Network::from_str("10.0.0.0/16")?]
        );
        assert!(deserialized_vnet.properties.subnets.is_empty());

        Ok(())
    }
}
