use crate::VirtualMachineId;
use crate::VirtualMachineName;
use crate::VirtualMachineProperties;
use arbitrary::Arbitrary;
use facet::Facet;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone, Arbitrary, Facet)]
pub struct VirtualMachine {
    pub id: VirtualMachineId,
    pub name: VirtualMachineName,
    pub location: Option<String>,
    #[facet(default, proxy = crate::StringMapDefaultNullProxy)]
    pub tags: HashMap<String, String>,
    pub properties: VirtualMachineProperties,
}

impl VirtualMachine {
    pub fn resource_group_id(&self) -> &crate::AzureResourceGroupId {
        &self.id.resource_group_id
    }
}

cloud_terrastodon_registry::register_thing!(VirtualMachine);
cloud_terrastodon_registry::register_arbitrary!(VirtualMachine);
