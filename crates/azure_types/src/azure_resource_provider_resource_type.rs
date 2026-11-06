use crate::AzureResourceProviderResourceTypeName;
use crate::azure_resource_provider_alias::AzureResourceProviderAlias;
use crate::azure_resource_provider_api_profile::AzureResourceProviderApiProfile;
use crate::azure_resource_provider_extended_location::AzureResourceProviderExtendedLocation;
use crate::azure_resource_provider_zone_mapping::AzureResourceProviderZoneMapping;
use arbitrary::Arbitrary;
use facet::Facet;
use std::collections::BTreeMap;

/// Metadata for a resource type advertised by a provider.
///
/// [Resource Management 2021-04-01 schema](https://learn.microsoft.com/en-us/rest/api/resources/providers/list-at-tenant-scope?view=rest-resources-2021-04-01#providerresourcetype).
/// The API schema does not require these fields; missing values remain explicit.
#[derive(Debug, Clone, PartialEq, Eq, Default, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureResourceProviderResourceType {
    pub resource_type: Option<AzureResourceProviderResourceTypeName>,
    pub locations: Option<Vec<String>>,
    pub location_mappings: Option<Vec<AzureResourceProviderExtendedLocation>>,
    pub aliases: Option<Vec<AzureResourceProviderAlias>>,
    pub api_versions: Option<Vec<String>>,
    pub default_api_version: Option<String>,
    pub zone_mappings: Option<Vec<AzureResourceProviderZoneMapping>>,
    pub api_profiles: Option<Vec<AzureResourceProviderApiProfile>>,
    pub capabilities: Option<String>,
    pub properties: Option<BTreeMap<String, String>>,
}

cloud_terrastodon_registry::register_thing!(AzureResourceProviderResourceType);
cloud_terrastodon_registry::register_arbitrary!(AzureResourceProviderResourceType);
