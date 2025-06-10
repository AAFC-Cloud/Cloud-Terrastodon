use crate::prelude::SubnetId;
use crate::prelude::SubnetProperties;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Subnet {
    pub id: Option<String>, // Full Azure resource ID
    pub name: String,       // This is the name from Azure, distinct from SubnetName in ID
    #[serde(rename = "type")]
    pub resource_type: Option<String>,
    pub etag: Option<String>,
    pub properties: Option<SubnetProperties>,
}

impl Subnet {
    pub fn subnet_id(&self) -> Option<eyre::Result<SubnetId>> {
        self.id.as_ref().map(|id| SubnetId::try_from(id.as_str()))
    }
}
