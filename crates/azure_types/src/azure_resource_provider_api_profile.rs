use arbitrary::Arbitrary;
/// Mapping between a provider API profile and an API version.
///
/// [Resource Management 2021-04-01 schema](https://learn.microsoft.com/en-us/rest/api/resources/providers/list-at-tenant-scope?view=rest-resources-2021-04-01#apiprofile).
/// The API schema does not require these fields; missing values remain explicit.
#[derive(Debug, Clone, PartialEq, Eq, Default, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureResourceProviderApiProfile {
    pub profile_version: Option<String>,
    pub api_version: Option<String>,
}

cloud_terrastodon_registry::register_thing!(AzureResourceProviderApiProfile);
cloud_terrastodon_registry::register_arbitrary!(AzureResourceProviderApiProfile);
