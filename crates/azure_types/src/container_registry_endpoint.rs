use crate::ContainerRegistry;
use crate::ContainerRegistryId;
use crate::ContainerRegistryLoginServer;
use arbitrary::Arbitrary;
use facet::Facet;

/// Resource Graph projection of the service-assigned registry login server.
/// [Registry properties](https://learn.microsoft.com/en-us/rest/api/container-registry/registries/get?view=rest-container-registry-2025-11-01#registry).
#[derive(Debug, Clone, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct ContainerRegistryEndpoint {
    pub id: ContainerRegistryId,
    pub login_server: ContainerRegistryLoginServer,
}

impl From<&ContainerRegistry> for ContainerRegistryEndpoint {
    /// Reuse the service-assigned login server from an already fetched registry.
    /// `properties.loginServer` has already been validated during decoding. The
    /// resource name is never substituted because it omits service DNS-label hashes.
    fn from(registry: &ContainerRegistry) -> Self {
        Self {
            id: registry.id.clone(),
            login_server: registry.properties.login_server.clone(),
        }
    }
}

cloud_terrastodon_registry::register_thing!(ContainerRegistryEndpoint);
cloud_terrastodon_registry::register_arbitrary!(ContainerRegistryEndpoint);
cloud_terrastodon_registry::register_fn_ref!(
    ContainerRegistry => ContainerRegistryEndpoint,
    kind = cloud_terrastodon_registry::FunctionKind::Conversion,
    label = "registry endpoint",
    trait_contract = cloud_terrastodon_registry::FunctionTraitContract::From,
    // The registry callback uses Result; From supplies this infallible blanket adapter.
    invoke = ContainerRegistryEndpoint::try_from,
);
