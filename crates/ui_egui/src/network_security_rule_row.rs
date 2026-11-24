use cloud_terrastodon_azure::AzureApplicationSecurityGroupReference;
use cloud_terrastodon_azure::AzureNetworkSecurityRule;
use cloud_terrastodon_azure::AzureNetworkSecurityRuleDirection;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::ScopeImpl;
use facet::Facet;

/// Display values and independently selectable matches for one security rule.
#[derive(Facet)]
pub(crate) struct NetworkSecurityRuleRow {
    pub id: ScopeImpl,
    pub name: String,
    pub direction: AzureNetworkSecurityRuleDirection,
    pub priority: u32,
    pub port: String,
    pub protocol: String,
    pub source: String,
    pub destination: String,
    pub action: String,
    pub ports: Vec<String>,
    pub sources: Vec<String>,
    pub destinations: Vec<String>,
}

impl From<&AzureNetworkSecurityRule> for NetworkSecurityRuleRow {
    fn from(rule: &AzureNetworkSecurityRule) -> Self {
        let properties = &rule.properties;
        let ports = display_values(
            properties.destination_port_range.as_deref(),
            properties.destination_port_ranges.as_deref(),
            None,
        );
        let sources = display_values(
            properties.source_address_prefix.as_deref(),
            properties.source_address_prefixes.as_deref(),
            properties.source_application_security_groups.as_deref(),
        );
        let destinations = display_values(
            properties.destination_address_prefix.as_deref(),
            properties.destination_address_prefixes.as_deref(),
            properties
                .destination_application_security_groups
                .as_deref(),
        );
        Self {
            id: rule.id.clone(),
            name: rule.name.clone(),
            direction: properties.direction.clone(),
            priority: u32::from(properties.priority),
            port: ports.join(", "),
            protocol: display_match(properties.protocol.as_str()).to_owned(),
            source: sources.join(", "),
            destination: destinations.join(", "),
            action: properties.access.as_str().to_owned(),
            ports,
            sources,
            destinations,
        }
    }
}

fn display_values(
    single: Option<&str>,
    multiple: Option<&[String]>,
    application_groups: Option<&[AzureApplicationSecurityGroupReference]>,
) -> Vec<String> {
    let mut values: Vec<_> = single
        .into_iter()
        .chain(multiple.into_iter().flatten().map(String::as_str))
        .filter(|value| !value.is_empty())
        .map(|value| display_match(value).to_owned())
        .chain(
            application_groups
                .into_iter()
                .flatten()
                .map(|group| group.id.expanded_form()),
        )
        .collect();
    values.sort_unstable();
    values.dedup();
    if values.is_empty() {
        values.push("—".to_owned());
    }
    values
}

fn display_match(value: &str) -> &str {
    match value {
        "*" => "Any",
        value => value,
    }
}
