use crate::ArbitraryJson;
use crate::ContainerRegistryLoginServer;
use arbitrary::Arbitrary;
use facet::Facet;
use std::collections::HashMap;

/// Registry properties, with the service-assigned endpoint decoded at the API boundary.
/// [Registry properties](https://learn.microsoft.com/en-us/rest/api/container-registry/registries/get?view=rest-container-registry-2025-11-01#registry).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct ContainerRegistryProperties {
    /// The actual login server, including any service-assigned DNS-label hash.
    /// A usable endpoint is required by this library's registry inventory contract.
    pub login_server: ContainerRegistryLoginServer,
    /// ARM properties retained while the former opaque properties model is
    /// strengthened incrementally. Consumers must promote properties to typed
    /// fields instead of parsing or indexing this map for operation inputs.
    #[facet(flatten)]
    #[arbitrary(default)]
    pub other: HashMap<String, ArbitraryJson>,
}

cloud_terrastodon_registry::register_thing!(ContainerRegistryProperties);
cloud_terrastodon_registry::register_arbitrary!(ContainerRegistryProperties);
