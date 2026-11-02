use arbitrary::Arbitrary;
/// Extended-location metadata advertised for an Azure location.
///
/// [Resource Management 2021-04-01 schema](https://learn.microsoft.com/en-us/rest/api/resources/providers/list-at-tenant-scope?view=rest-resources-2021-04-01#providerextendedlocation).
/// The API schema does not require these fields; missing values remain explicit.
#[derive(Debug, Clone, PartialEq, Eq, Default, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureResourceProviderExtendedLocation {
    pub location: Option<String>,
    #[facet(rename = "type")]
    pub kind: Option<String>,
    pub extended_locations: Option<Vec<String>>,
}

cloud_terrastodon_registry::register_thing!(AzureResourceProviderExtendedLocation);
cloud_terrastodon_registry::register_arbitrary!(AzureResourceProviderExtendedLocation);
