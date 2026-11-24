use arbitrary::Arbitrary;
use facet::Facet;

/// Whether a network security rule applies to inbound or outbound traffic.
/// [Azure Network 2026-01-01 schema](https://learn.microsoft.com/en-us/rest/api/virtualnetwork/network-security-groups/get?view=rest-virtualnetwork-2026-01-01#securityruledirection).
#[derive(Debug, Clone, Eq, PartialEq, Hash, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureNetworkSecurityRuleDirection {
    Inbound,
    Outbound,
    #[arbitrary(skip)]
    Unknown(String),
}

impl AzureNetworkSecurityRuleDirection {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Inbound => "Inbound",
            Self::Outbound => "Outbound",
            Self::Unknown(value) => value,
        }
    }
}

impl From<String> for AzureNetworkSecurityRuleDirection {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Inbound" => Self::Inbound,
            "Outbound" => Self::Outbound,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureNetworkSecurityRuleDirection> for String {
    fn from(value: &AzureNetworkSecurityRuleDirection) -> Self {
        value.as_str().to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureNetworkSecurityRuleDirection);
cloud_terrastodon_registry::register_arbitrary!(AzureNetworkSecurityRuleDirection);
