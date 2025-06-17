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
    #[serde(deserialize_with = "crate::serde_helpers::deserialize_null_default")]
    pub tags: HashMap<String, String>,
    pub properties: VirtualNetworkProperties,
}

impl VirtualNetwork {
    // Helper to get the ResourceGroupId from the VirtualNetworkId
    pub fn resource_group_id(&self) -> &ResourceGroupId {
        &self.id.resource_group_id
    }
}
