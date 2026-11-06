use crate::azure_resource_provider_alias_pattern_type::AzureResourceProviderAliasPatternType;
use arbitrary::Arbitrary;
use facet::Facet;

/// Pattern-based interpretation of a policy-alias path.
///
/// [Resource Management 2021-04-01 schema](https://learn.microsoft.com/en-us/rest/api/resources/providers/list-at-tenant-scope?view=rest-resources-2021-04-01#aliaspattern).
/// The API schema does not require these fields; missing values remain explicit.
#[derive(Debug, Clone, PartialEq, Eq, Default, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureResourceProviderAliasPattern {
    pub phrase: Option<String>,
    pub variable: Option<String>,
    #[facet(rename = "type")]
    pub kind: Option<AzureResourceProviderAliasPatternType>,
}

cloud_terrastodon_registry::register_thing!(AzureResourceProviderAliasPattern);
cloud_terrastodon_registry::register_arbitrary!(AzureResourceProviderAliasPattern);
