use crate::prelude::ResourceGroupId;
use crate::prelude::SubscriptionId;
use crate::prelude::VirtualNetworkId;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct VirtualNetwork {
    pub id: VirtualNetworkId,
    pub name: String, // This is the name from Azure, distinct from VirtualNetworkName in ID
    pub location: String,
    pub tags: Option<HashMap<String, String>>,
    // Add other relevant fields from the Azure API response for Virtual Network
    // For example:
    // pub properties: VirtualNetworkProperties,
}

// Placeholder for actual properties if needed
// #[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
// pub struct VirtualNetworkProperties {
//     #[serde(rename = "addressSpace")]
//     pub address_space: Option<AddressSpace>,
//     pub subnets: Option<Vec<Subnet>>,
//     // Add other properties as needed
// }

// #[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
// pub struct AddressSpace {
//     #[serde(rename = "addressPrefixes")]
//     pub address_prefixes: Option<Vec<String>>,
// }

// #[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
// pub struct Subnet {
//     pub id: Option<String>, // Subnet ID
//     pub name: Option<String>,
//     // Add other subnet properties as needed
// }

impl VirtualNetwork {
    // Helper to get the ResourceGroupId from the VirtualNetworkId
    pub fn resource_group_id(&self) -> &ResourceGroupId {
        &self.id.resource_group_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::ResourceGroupName;
    use crate::prelude::VirtualNetworkName;
    use crate::scopes::Scope;
    use crate::slug::Slug;
    use eyre::Result;
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

        Ok(())
    }
}
