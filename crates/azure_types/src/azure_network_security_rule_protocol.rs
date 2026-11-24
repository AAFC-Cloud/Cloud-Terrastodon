use arbitrary::Arbitrary;
use facet::Facet;

/// The transport protocol evaluated by a network security rule.
/// [Azure Network 2026-01-01 schema](https://learn.microsoft.com/en-us/rest/api/virtualnetwork/network-security-groups/get?view=rest-virtualnetwork-2026-01-01#securityruleprotocol).
#[derive(Debug, Clone, Eq, PartialEq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureNetworkSecurityRuleProtocol {
    Any,
    Tcp,
    Udp,
    Icmp,
    Esp,
    Ah,
    #[arbitrary(skip)]
    Unknown(String),
}

impl AzureNetworkSecurityRuleProtocol {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Any => "*",
            Self::Tcp => "Tcp",
            Self::Udp => "Udp",
            Self::Icmp => "Icmp",
            Self::Esp => "Esp",
            Self::Ah => "Ah",
            Self::Unknown(value) => value,
        }
    }
}

impl From<String> for AzureNetworkSecurityRuleProtocol {
    fn from(value: String) -> Self {
        match value.as_str() {
            "*" => Self::Any,
            "Tcp" => Self::Tcp,
            "Udp" => Self::Udp,
            "Icmp" => Self::Icmp,
            "Esp" => Self::Esp,
            "Ah" => Self::Ah,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureNetworkSecurityRuleProtocol> for String {
    fn from(value: &AzureNetworkSecurityRuleProtocol) -> Self {
        value.as_str().to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureNetworkSecurityRuleProtocol);
cloud_terrastodon_registry::register_arbitrary!(AzureNetworkSecurityRuleProtocol);
