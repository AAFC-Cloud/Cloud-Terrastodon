use crate::ContainerRegistryId;
use crate::ContainerRegistryLoginServer;
use arbitrary::Arbitrary;

/// Resource Graph projection of the service-assigned registry login server.
/// [Registry properties](https://learn.microsoft.com/en-us/rest/api/container-registry/registries/get?view=rest-container-registry-2025-11-01#registry).
#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct ContainerRegistryEndpoint {
    pub id: ContainerRegistryId,
    pub login_server: ContainerRegistryLoginServer,
}
cloud_terrastodon_registry::register_thing!(ContainerRegistryEndpoint);
cloud_terrastodon_registry::register_arbitrary!(ContainerRegistryEndpoint);
