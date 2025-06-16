use crate::prelude::SubnetId;
use crate::prelude::SubnetName;
use crate::prelude::SubnetProperties;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Subnet {
    pub id: SubnetId,
    pub name: SubnetName,
    pub properties: SubnetProperties,
}
