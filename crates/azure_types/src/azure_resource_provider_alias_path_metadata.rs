use crate::azure_resource_provider_alias_path_attributes::AzureResourceProviderAliasPathAttributes;
use crate::azure_resource_provider_alias_path_token_type::AzureResourceProviderAliasPathTokenType;
use arbitrary::Arbitrary;
use facet::Facet;

/// Type and policy-modification metadata for a policy-alias path.
///
/// [Resource Management 2021-04-01 schema](https://learn.microsoft.com/en-us/rest/api/resources/providers/list-at-tenant-scope?view=rest-resources-2021-04-01#aliaspathmetadata).
/// The API schema does not require these fields; missing values remain explicit.
#[derive(Debug, Clone, PartialEq, Eq, Default, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureResourceProviderAliasPathMetadata {
    #[facet(rename = "type")]
    pub kind: Option<AzureResourceProviderAliasPathTokenType>,
    pub attributes: Option<AzureResourceProviderAliasPathAttributes>,
}

cloud_terrastodon_registry::register_thing!(AzureResourceProviderAliasPathMetadata);
cloud_terrastodon_registry::register_arbitrary!(AzureResourceProviderAliasPathMetadata);
