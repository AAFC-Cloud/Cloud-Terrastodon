use crate::SubnetId;
use crate::SubnetName;
use crate::scopes::Scope;
use arbitrary::Arbitrary;
use eyre::WrapErr;
use std::fmt::Display;
use std::str::FromStr;

/// Select a subnet by its complete Azure resource ID or validated name.
///
/// A leading `/` selects resource-ID parsing, so malformed IDs retain their
/// original parsing error instead of becoming name selectors. Names follow the
/// [Microsoft.Network naming rules](https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/resource-name-rules#microsoftnetwork).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureSubnetArgument {
    Id(SubnetId),
    Name(SubnetName),
}

impl AzureSubnetArgument {
    /// Match the complete resource identity or name without regard to ASCII case.
    ///
    /// Name selectors can match subnets in multiple parent virtual networks
    /// within the selected inventory.
    pub fn matches_id(&self, id: &SubnetId) -> bool {
        match self {
            Self::Id(selected) => selected
                .expanded_form()
                .eq_ignore_ascii_case(&id.expanded_form()),
            Self::Name(name) => name.as_str().eq_ignore_ascii_case(id.subnet_name.as_str()),
        }
    }
}

impl FromStr for AzureSubnetArgument {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.starts_with('/') {
            value
                .parse::<SubnetId>()
                .map(Self::Id)
                .wrap_err("Invalid subnet resource ID selector")
        } else {
            value.parse::<SubnetName>().map(Self::Name)
        }
    }
}

impl Display for AzureSubnetArgument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Id(id) => id.fmt(f),
            Self::Name(name) => name.fmt(f),
        }
    }
}

impl From<SubnetId> for AzureSubnetArgument {
    fn from(value: SubnetId) -> Self {
        Self::Id(value)
    }
}

impl From<SubnetName> for AzureSubnetArgument {
    fn from(value: SubnetName) -> Self {
        Self::Name(value)
    }
}

crate::impl_facet_string_proxy!(AzureSubnetArgument, value => value.to_string());
cloud_terrastodon_registry::register_thing!(AzureSubnetArgument);
cloud_terrastodon_registry::register_arbitrary!(AzureSubnetArgument);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ResourceGroupId;
    use crate::ResourceGroupName;
    use crate::SubscriptionId;
    use crate::VirtualNetworkId;
    use crate::VirtualNetworkName;
    use uuid::Uuid;

    fn subnet_id(network: &str, name: &str) -> SubnetId {
        SubnetId::new(
            VirtualNetworkId::new(
                ResourceGroupId::new(
                    SubscriptionId::new(Uuid::nil()),
                    "ExampleGroup".parse::<ResourceGroupName>().unwrap(),
                ),
                network.parse::<VirtualNetworkName>().unwrap(),
            ),
            name.parse::<SubnetName>().unwrap(),
        )
    }

    #[test]
    fn name_selectors_preserve_domain_validation_and_resource_words() {
        for name in ["subnets", "a"] {
            assert!(matches!(
                name.parse::<AzureSubnetArgument>().unwrap(),
                AzureSubnetArgument::Name(_)
            ));
        }
        assert!("".parse::<AzureSubnetArgument>().is_err());
        assert!("subnet/child".parse::<AzureSubnetArgument>().is_err());
    }

    #[test]
    fn malformed_resource_id_preserves_the_subscription_parse_error() {
        let error = "/subscriptions/not-a-guid/resourceGroups/example/providers/Microsoft.Network/virtualNetworks/example/subnets/example"
            .parse::<AzureSubnetArgument>()
            .unwrap_err();
        assert!(error.downcast_ref::<uuid::Error>().is_some());
    }

    #[test]
    fn id_selectors_distinguish_parent_networks_and_names_do_not() {
        let first = subnet_id("FirstNetwork", "ExampleSubnet");
        let second = subnet_id("SecondNetwork", "examplesubnet");
        let selected = first
            .expanded_form()
            .to_ascii_uppercase()
            .parse::<AzureSubnetArgument>()
            .unwrap();
        assert!(selected.matches_id(&first));
        assert!(!selected.matches_id(&second));

        let selected = "EXAMPLESUBNET".parse::<AzureSubnetArgument>().unwrap();
        assert!(selected.matches_id(&first));
        assert!(selected.matches_id(&second));
    }
}
