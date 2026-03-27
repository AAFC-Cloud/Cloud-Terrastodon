use crate::SubnetId;
use crate::SubnetName;
use crate::SubnetProperties;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Subnet {
    pub id: SubnetId,
    pub name: SubnetName,
    pub properties: SubnetProperties,
}
