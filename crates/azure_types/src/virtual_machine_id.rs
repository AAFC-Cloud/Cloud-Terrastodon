use crate::prelude::ResourceGroupId;
use crate::prelude::VirtualMachineName;
use crate::scopes::HasPrefix;
use crate::scopes::NameValidatable;
use crate::scopes::Scope;
use crate::scopes::ScopeImplKind;
use crate::scopes::TryFromResourceGroupScoped;
use crate::slug::HasSlug;
use crate::slug::Slug;
use arbitrary::Arbitrary;
use compact_str::ToCompactString;
use eyre::Context;
use eyre::Result;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use std::str::FromStr;

pub const VIRTUAL_MACHINE_ID_PREFIX: &str = "/providers/Microsoft.Compute/virtualMachines/";

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, Arbitrary)]
pub struct VirtualMachineId {
    pub resource_group_id: ResourceGroupId,
    pub vm_name: VirtualMachineName,
}
impl core::fmt::Display for VirtualMachineId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.resource_group_id,
            VIRTUAL_MACHINE_ID_PREFIX,
            self.vm_name
        )
    }
}

impl VirtualMachineId {
    pub fn new(resource_group_id: ResourceGroupId, vm_name: impl Into<VirtualMachineName>) -> Self {
        Self {
            resource_group_id,
            vm_name: vm_name.into(),
        }
    }

    pub fn try_new<N>(resource_group_id: ResourceGroupId, vm_name: N) -> Result<Self>
    where
        N: TryInto<VirtualMachineName>,
        N::Error: Into<eyre::Error>,
    {
        Ok(Self {
            resource_group_id,
            vm_name: vm_name
                .try_into()
                .map_err(Into::into)
                .context("Failed to convert to VirtualMachineName")?,
        })
    }
}

impl HasSlug for VirtualMachineId {
    type Name = VirtualMachineName;

    fn name(&self) -> &Self::Name {
        &self.vm_name
    }
}

impl AsRef<ResourceGroupId> for VirtualMachineId {
    fn as_ref(&self) -> &ResourceGroupId {
        &self.resource_group_id
    }
}

impl AsRef<VirtualMachineName> for VirtualMachineId {
    fn as_ref(&self) -> &VirtualMachineName {
        &self.vm_name
    }
}

impl NameValidatable for VirtualMachineId {
    fn validate_name(name: &str) -> Result<()> {
        VirtualMachineName::try_new(name).map(|_| ())
    }
}

impl HasPrefix for VirtualMachineId {
    fn get_prefix() -> &'static str {
        VIRTUAL_MACHINE_ID_PREFIX
    }
}

impl TryFromResourceGroupScoped for VirtualMachineId {
    unsafe fn new_resource_group_scoped_unchecked(
        _expanded: &str,
        resource_group_id: ResourceGroupId,
        name: Self::Name,
    ) -> Self {
        Self {
            resource_group_id,
            vm_name: name,
        }
    }
}

impl Scope for VirtualMachineId {
    fn try_from_expanded(expanded: &str) -> Result<Self> {
        VirtualMachineId::try_from_expanded_resource_group_scoped(expanded)
    }

    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.resource_group_id.expanded_form(),
            VIRTUAL_MACHINE_ID_PREFIX,
            self.vm_name
        )
    }

    fn kind(&self) -> ScopeImplKind {
        // There is no specialized ScopeImplKind for VMs; treat as resource
        ScopeImplKind::Unknown
    }

    fn as_scope_impl(&self) -> crate::scopes::ScopeImpl {
        // There is no VirtualMachine variant on ScopeImpl; fall back to Resource via ResourceId parsing
        crate::scopes::ScopeImpl::Unknown(self.expanded_form().to_compact_string())
    }
}

impl FromStr for VirtualMachineId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        VirtualMachineId::try_from_expanded(s)
    }
}

impl Serialize for VirtualMachineId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.expanded_form().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for VirtualMachineId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::try_from_expanded(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::ResourceGroupName;
    use crate::prelude::SubscriptionId;
    use std::str::FromStr;

    #[test]
    fn test_virtual_machine_id_serialization_deserialization() -> eyre::Result<()> {
        let sub_id = SubscriptionId::from_str("00000000-0000-0000-0000-000000000000")?;
        let rg_id = ResourceGroupId::new(sub_id, ResourceGroupName::try_new("test-rg").unwrap());
        let vm_id = VirtualMachineId::try_new(rg_id, "test-vm")?;

        let serialized = serde_json::to_string(&vm_id)?;
        let expected_str = "/subscriptions/00000000-0000-0000-0000-000000000000/resourceGroups/test-rg/providers/Microsoft.Compute/virtualMachines/test-vm".to_string();
        assert_eq!(serialized, serde_json::to_string(&expected_str)?);

        let deserialized: VirtualMachineId = serde_json::from_str(&serialized)?;
        assert_eq!(vm_id, deserialized);
        Ok(())
    }
}
