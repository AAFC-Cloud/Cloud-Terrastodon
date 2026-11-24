use crate::network_security_rule_row::NetworkSecurityRuleRow;
use cloud_terrastodon_azure::AzureNetworkSecurityGroupProperties;

/// A table projection of the available rule collections in a network security group snapshot.
pub(crate) struct NetworkSecurityGroupRuleRows {
    pub rows: Vec<NetworkSecurityRuleRow>,
    pub custom_available: bool,
    pub defaults_available: bool,
}

impl From<&AzureNetworkSecurityGroupProperties> for NetworkSecurityGroupRuleRows {
    fn from(properties: &AzureNetworkSecurityGroupProperties) -> Self {
        Self {
            custom_available: properties.security_rules.is_some(),
            defaults_available: properties.default_security_rules.is_some(),
            rows: properties
                .security_rules
                .iter()
                .flatten()
                .chain(properties.default_security_rules.iter().flatten())
                .map(NetworkSecurityRuleRow::from)
                .collect(),
        }
    }
}
