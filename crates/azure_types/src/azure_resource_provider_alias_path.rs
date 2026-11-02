use crate::azure_resource_provider_alias_path_metadata::AzureResourceProviderAliasPathMetadata;
use crate::azure_resource_provider_alias_pattern::AzureResourceProviderAliasPattern;
use arbitrary::Arbitrary;

/// An API-version-specific policy-alias path.
///
/// [Resource Management 2021-04-01 schema](https://learn.microsoft.com/en-us/rest/api/resources/providers/list-at-tenant-scope?view=rest-resources-2021-04-01#aliaspath).
/// The API schema does not require these fields; missing values remain explicit.
#[derive(Debug, Clone, PartialEq, Eq, Default, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureResourceProviderAliasPath {
    pub path: Option<String>,
    pub api_versions: Option<Vec<String>>,
    pub pattern: Option<AzureResourceProviderAliasPattern>,
    pub metadata: Option<AzureResourceProviderAliasPathMetadata>,
}

cloud_terrastodon_registry::register_thing!(AzureResourceProviderAliasPath);
cloud_terrastodon_registry::register_arbitrary!(AzureResourceProviderAliasPath);
