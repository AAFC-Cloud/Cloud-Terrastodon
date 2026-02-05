use crate::prelude::VirtualMachineId;
use crate::prelude::VirtualMachineName;
use crate::prelude::VirtualMachineProperties;
use crate::serde_helpers::deserialize_default_if_null;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct VirtualMachine {
    pub id: VirtualMachineId,
    pub name: VirtualMachineName,
    pub location: Option<String>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub tags: HashMap<String, String>,
    pub properties: VirtualMachineProperties,
}

impl VirtualMachine {
    pub fn resource_group_id(&self) -> &crate::prelude::ResourceGroupId {
        &self.id.resource_group_id
    }
}
