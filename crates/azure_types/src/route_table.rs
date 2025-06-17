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
