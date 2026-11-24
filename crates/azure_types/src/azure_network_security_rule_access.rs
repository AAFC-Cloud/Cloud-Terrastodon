use arbitrary::Arbitrary;
use facet::Facet;

/// What the rule does to traffic that matches its conditions.
/// [Azure Network 2026-01-01 schema](https://learn.microsoft.com/en-us/rest/api/virtualnetwork/network-security-groups/get?view=rest-virtualnetwork-2026-01-01#securityruleaccess).
#[derive(Debug, Clone, Eq, PartialEq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureNetworkSecurityRuleAccess {
    Allow,
    Deny,
    #[arbitrary(skip)]
    Unknown(String),
}

impl AzureNetworkSecurityRuleAccess {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Allow => "Allow",
            Self::Deny => "Deny",
            Self::Unknown(value) => value,
        }
    }
}

impl From<String> for AzureNetworkSecurityRuleAccess {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Allow" => Self::Allow,
            "Deny" => Self::Deny,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureNetworkSecurityRuleAccess> for String {
    fn from(value: &AzureNetworkSecurityRuleAccess) -> Self {
        value.as_str().to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureNetworkSecurityRuleAccess);
cloud_terrastodon_registry::register_arbitrary!(AzureNetworkSecurityRuleAccess);
