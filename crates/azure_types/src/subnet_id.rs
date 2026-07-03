use crate::SubnetName;
use crate::VirtualNetworkId;
use crate::scopes::Scope;
use crate::scopes::ScopeImpl;
use crate::scopes::TryFromVirtualNetworkScoped;
use crate::slug::Slug;
use eyre::Context;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, facet::Facet)]
#[facet(json::proxy = String)]
pub struct SubnetId {
    pub virtual_network_id: VirtualNetworkId,
    pub subnet_name: SubnetName,
}
crate::impl_facet_string_proxy!(SubnetId, value => value.expanded_form());

impl SubnetId {
    pub fn new(
        virtual_network_id: impl Into<VirtualNetworkId>,
        subnet_name: impl Into<SubnetName>,
    ) -> Self {
        Self {
            virtual_network_id: virtual_network_id.into(),
            subnet_name: subnet_name.into(),
        }
    }

    pub fn try_new<V, N>(virtual_network_id: V, subnet_name: N) -> eyre::Result<Self>
    where
        V: TryInto<VirtualNetworkId>,
        V::Error: Into<eyre::Error>,
        N: TryInto<SubnetName>,
        N::Error: Into<eyre::Error>,
    {
        let virtual_network_id = virtual_network_id
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert virtual_network_id")?;
        let subnet_name = subnet_name
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert subnet_name")?;
        Ok(Self {
            virtual_network_id,
            subnet_name,
        })
    }
}

impl Scope for SubnetId {
    type Err = <Self as std::str::FromStr>::Err;
    fn expanded_form(&self) -> String {
        format!(
            "{}/subnets/{}",
            self.virtual_network_id.expanded_form(),
            self.subnet_name
        )
    }

    fn short_form(&self) -> String {
        format!(
            "{}/{}",
            self.virtual_network_id.short_form(),
            self.subnet_name
        )
    }

    fn try_from_expanded(expanded: &str) -> eyre::Result<Self> {
        // Parse subnet ID format: /subscriptions/{subId}/resourceGroups/{rgName}/providers/Microsoft.Network/virtualNetworks/{vnetName}/subnets/{subnetName}

        // Find the last "/subnets/" occurrence
        if let Some(subnets_pos) = expanded.rfind("/subnets/") {
            let vnet_part = &expanded[..subnets_pos];
            let subnet_name_part = &expanded[subnets_pos + "/subnets/".len()..];

            let virtual_network_id = VirtualNetworkId::try_from_expanded(vnet_part)?;
            let subnet_name = SubnetName::try_new(subnet_name_part)?;

            Ok(Self::new(virtual_network_id, subnet_name))
        } else {
            Err(eyre::eyre!("Invalid subnet ID format: {}", expanded))
        }
    }

    fn as_scope_impl(&self) -> ScopeImpl {
        ScopeImpl::Subnet(self.clone())
    }

    fn kind(&self) -> crate::scopes::ScopeImplKind {
        crate::scopes::ScopeImplKind::Subnet
    }
}

impl FromStr for SubnetId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        SubnetId::try_from_expanded(s)
    }
}

impl TryFromVirtualNetworkScoped for SubnetId {
    fn try_from_virtual_network_scoped(
        virtual_network_id: &VirtualNetworkId,
        name: &str,
    ) -> eyre::Result<Self> {
        let subnet_name = SubnetName::try_new(name)?;
        Ok(Self::new(virtual_network_id.clone(), subnet_name))
    }
}

impl TryFrom<&str> for SubnetId {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from_expanded(value)
    }
}

impl TryFrom<ScopeImpl> for SubnetId {
    type Error = eyre::Error;

    fn try_from(scope: ScopeImpl) -> Result<Self, Self::Error> {
        match scope {
            ScopeImpl::Subnet(subnet_id) => Ok(subnet_id),
            _ => Err(eyre::eyre!("Expected Subnet scope, got {:?}", scope)),
        }
    }
}

impl From<SubnetId> for ScopeImpl {
    fn from(subnet_id: SubnetId) -> Self {
        ScopeImpl::Subnet(subnet_id)
    }
}

impl Display for SubnetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expanded_form())
    }
}

cloud_terrastodon_registry::register_thing!(SubnetId);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ResourceGroupId;
    use crate::ResourceGroupName;
    use crate::SubscriptionId;
    use crate::VirtualNetworkName;
    use uuid::Uuid;

    #[test]
    fn test_subnet_id_creation_and_scopes() -> eyre::Result<()> {
        let sub_id = SubscriptionId::new(Uuid::new_v4());
        let rg_name = ResourceGroupName::try_new("test-rg")?;
        let rg_id = ResourceGroupId::new(sub_id, rg_name);
        let vnet_name = VirtualNetworkName::try_new("test-vnet")?;
        let vnet_id = VirtualNetworkId::new(rg_id, vnet_name);
        let subnet_name = SubnetName::try_new("test-subnet")?;
        let subnet_id = SubnetId::new(vnet_id.clone(), subnet_name.clone());

        // Test expanded form
        let expected_expanded = format!("{}/subnets/{}", vnet_id.expanded_form(), subnet_name);
        assert_eq!(subnet_id.expanded_form(), expected_expanded);

        // Test short form
        let expected_short = format!("{}/{}", vnet_id.short_form(), subnet_name);
        assert_eq!(subnet_id.short_form(), expected_short);

        // Test round-trip through string
        let subnet_id_from_str = SubnetId::try_from(subnet_id.expanded_form().as_str())?;
        assert_eq!(subnet_id, subnet_id_from_str);
        let json = facet_json::to_string(&subnet_id.expanded_form())?;
        assert_eq!(facet_json::to_string(&subnet_id)?, json);
        assert_eq!(facet_json::from_str::<SubnetId>(&json)?, subnet_id);

        Ok(())
    }
}

