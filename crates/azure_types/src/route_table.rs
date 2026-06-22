use crate::ResourceGroupId;
use crate::RouteTableId;
use crate::RouteTableProperties;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone, facet::Facet)]
pub struct RouteTable {
    pub id: RouteTableId,
    pub name: String, // This is the name from Azure, distinct from RouteTableName in ID
    pub location: String,
    #[facet(default, opaque, proxy = crate::StringMapDefaultNullProxy)]
    pub tags: HashMap<String, String>,
    pub properties: RouteTableProperties,
}

impl RouteTable {
    // Helper to get the ResourceGroupId from the RouteTableId
    pub fn resource_group_id(&self) -> &ResourceGroupId {
        &self.id.resource_group_id
    }
}
