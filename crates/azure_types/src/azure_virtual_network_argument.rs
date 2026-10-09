use crate::VirtualNetworkId;
use crate::VirtualNetworkName;
use crate::scopes::Scope;
use arbitrary::Arbitrary;
use eyre::WrapErr;
use std::fmt::Display;
use std::str::FromStr;

/// Select a virtual network by its complete Azure resource ID or validated name.
///
/// A leading `/` selects resource-ID parsing, so malformed IDs retain their
/// original parsing error instead of becoming name selectors. Names follow the
/// [Microsoft.Network naming rules](https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/resource-name-rules#microsoftnetwork).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureVirtualNetworkArgument {
    Id(VirtualNetworkId),
    Name(VirtualNetworkName),
}

impl AzureVirtualNetworkArgument {
    /// Match the complete resource identity or name without regard to ASCII case.
    ///
    /// Name selectors can match networks in multiple subscriptions or resource
    /// groups within the selected inventory.
    pub fn matches_id(&self, id: &VirtualNetworkId) -> bool {
        match self {
            Self::Id(selected) => selected
                .expanded_form()
                .eq_ignore_ascii_case(&id.expanded_form()),
            Self::Name(name) => name
                .as_str()
                .eq_ignore_ascii_case(id.virtual_network_name.as_str()),
        }
    }
}

impl FromStr for AzureVirtualNetworkArgument {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.starts_with('/') {
            value
                .parse::<VirtualNetworkId>()
                .map(Self::Id)
                .wrap_err("Invalid virtual network resource ID selector")
        } else {
            value.parse::<VirtualNetworkName>().map(Self::Name)
        }
    }
}

impl Display for AzureVirtualNetworkArgument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Id(id) => f.write_str(&id.expanded_form()),
            Self::Name(name) => name.fmt(f),
        }
    }
}

impl From<VirtualNetworkId> for AzureVirtualNetworkArgument {
    fn from(value: VirtualNetworkId) -> Self {
        Self::Id(value)
    }
}

impl From<VirtualNetworkName> for AzureVirtualNetworkArgument {
    fn from(value: VirtualNetworkName) -> Self {
        Self::Name(value)
    }
}

crate::impl_facet_string_proxy!(AzureVirtualNetworkArgument, value => value.to_string());
cloud_terrastodon_registry::register_thing!(AzureVirtualNetworkArgument);
cloud_terrastodon_registry::register_arbitrary!(AzureVirtualNetworkArgument);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ResourceGroupId;
    use crate::ResourceGroupName;
    use crate::SubscriptionId;
    use uuid::Uuid;

    fn network_id(resource_group: &str, name: &str) -> VirtualNetworkId {
        VirtualNetworkId::new(
            ResourceGroupId::new(
                SubscriptionId::new(Uuid::nil()),
                resource_group.parse::<ResourceGroupName>().unwrap(),
            ),
            name.parse::<VirtualNetworkName>().unwrap(),
        )
    }

    #[test]
    fn name_selectors_preserve_domain_validation_and_resource_words() {
        assert!(matches!(
            "subscriptions"
                .parse::<AzureVirtualNetworkArgument>()
                .unwrap(),
            AzureVirtualNetworkArgument::Name(_)
        ));
        assert!("a".parse::<AzureVirtualNetworkArgument>().is_err());
        assert!(
            "network/child"
                .parse::<AzureVirtualNetworkArgument>()
                .is_err()
        );
    }

    #[test]
    fn malformed_resource_id_preserves_the_subscription_parse_error() {
        let error = "/subscriptions/not-a-guid/resourceGroups/example/providers/Microsoft.Network/virtualNetworks/example"
            .parse::<AzureVirtualNetworkArgument>()
            .unwrap_err();
        assert!(error.downcast_ref::<uuid::Error>().is_some());
    }

    #[test]
    fn id_selectors_are_scoped_and_names_can_match_multiple_scopes() {
        let first = network_id("FirstGroup", "ExampleNetwork");
        let second = network_id("SecondGroup", "examplenetwork");
        let selected = first
            .expanded_form()
            .to_ascii_uppercase()
            .parse::<AzureVirtualNetworkArgument>()
            .unwrap();
        assert!(selected.matches_id(&first));
        assert!(!selected.matches_id(&second));

        let selected = "EXAMPLENETWORK"
            .parse::<AzureVirtualNetworkArgument>()
            .unwrap();
        assert!(selected.matches_id(&first));
        assert!(selected.matches_id(&second));
    }
}
