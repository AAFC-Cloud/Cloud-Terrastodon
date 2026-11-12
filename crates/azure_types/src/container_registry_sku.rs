use arbitrary::Arbitrary;
use facet::Facet;

/// The SKU name and pricing tier returned for a container registry.
/// [Registry SKU](https://learn.microsoft.com/en-us/rest/api/container-registry/registries/get?view=rest-container-registry-2025-11-01#sku).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct ContainerRegistrySKU {
    pub name: String,
    pub tier: String,
}

cloud_terrastodon_registry::register_thing!(ContainerRegistrySKU);
cloud_terrastodon_registry::register_arbitrary!(ContainerRegistrySKU);
