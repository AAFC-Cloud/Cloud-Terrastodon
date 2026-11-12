use crate::ArbitraryJson;
use crate::ContainerRegistry;
use crate::ContainerRegistryId;
use crate::ContainerRegistryLoginServer;
use arbitrary::Arbitrary;
use eyre::Context;
use eyre::Result;
use facet::Facet;
use std::collections::HashMap;

/// Resource Graph projection of the service-assigned registry login server.
/// [Registry properties](https://learn.microsoft.com/en-us/rest/api/container-registry/registries/get?view=rest-container-registry-2025-11-01#registry).
#[derive(Debug, Clone, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct ContainerRegistryEndpoint {
    pub id: ContainerRegistryId,
    pub login_server: ContainerRegistryLoginServer,
}

impl TryFrom<&ContainerRegistry> for ContainerRegistryEndpoint {
    type Error = eyre::Report;

    /// Reuse the service-assigned login server from an already fetched registry.
    /// Missing or invalid `properties.loginServer` is an error; the resource name
    /// is never substituted because it does not include service DNS-label hashes.
    fn try_from(registry: &ContainerRegistry) -> Result<Self> {
        let properties: HashMap<String, ArbitraryJson> =
            facet_json::from_str(registry.properties.as_ref())
                .wrap_err("container registry properties must be an object")?;
        let login_server = properties
            .get("loginServer")
            .ok_or_else(|| eyre::eyre!("container registry properties omitted loginServer"))?;
        Ok(Self {
            id: registry.id.clone(),
            login_server: facet_json::from_str(login_server.as_ref())
                .wrap_err("container registry properties contain an invalid loginServer")?,
        })
    }
}

cloud_terrastodon_registry::register_thing!(ContainerRegistryEndpoint);
cloud_terrastodon_registry::register_arbitrary!(ContainerRegistryEndpoint);
cloud_terrastodon_registry::register_fn_ref!(
    ContainerRegistry => ContainerRegistryEndpoint,
    kind = cloud_terrastodon_registry::FunctionKind::Conversion,
    label = "registry endpoint",
    trait_contract = cloud_terrastodon_registry::FunctionTraitContract::TryFrom,
    invoke = ContainerRegistryEndpoint::try_from,
);
