use crate::prelude::ResourceGroupId;
use crate::prelude::VirtualNetworkId;
use crate::prelude::VirtualNetworkName;
use crate::prelude::VirtualNetworkProperties;
use crate::serde_helpers::deserialize_default_if_null;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct VirtualNetwork {
    pub id: VirtualNetworkId,
    pub name: VirtualNetworkName,
    pub location: String,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub tags: HashMap<String, String>,
    pub properties: VirtualNetworkProperties,
}

impl VirtualNetwork {
    // Helper to get the ResourceGroupId from the VirtualNetworkId
    pub fn resource_group_id(&self) -> &ResourceGroupId {
        &self.id.resource_group_id
    }
}
