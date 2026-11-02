use crate::azure_resource_provider_alias_path::AzureResourceProviderAliasPath;
use crate::azure_resource_provider_alias_path_metadata::AzureResourceProviderAliasPathMetadata;
use crate::azure_resource_provider_alias_pattern::AzureResourceProviderAliasPattern;
use crate::azure_resource_provider_alias_type::AzureResourceProviderAliasType;
use arbitrary::Arbitrary;

/// Policy-alias paths and their default interpretation.
///
/// [Resource Management 2021-04-01 schema](https://learn.microsoft.com/en-us/rest/api/resources/providers/list-at-tenant-scope?view=rest-resources-2021-04-01#alias).
/// The API schema does not require these fields; missing values remain explicit.
#[derive(Debug, Clone, PartialEq, Eq, Default, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureResourceProviderAlias {
    pub name: Option<String>,
    pub paths: Option<Vec<AzureResourceProviderAliasPath>>,
    #[facet(rename = "type")]
    pub kind: Option<AzureResourceProviderAliasType>,
    pub default_path: Option<String>,
    pub default_pattern: Option<AzureResourceProviderAliasPattern>,
    pub default_metadata: Option<AzureResourceProviderAliasPathMetadata>,
}

cloud_terrastodon_registry::register_thing!(AzureResourceProviderAlias);
cloud_terrastodon_registry::register_arbitrary!(AzureResourceProviderAlias);
