use arbitrary::Arbitrary;

/// [Tag changeable attributes, API 2021-07-01](https://learn.microsoft.com/en-us/rest/api/registry-dataplane/container-registry/get-tags?view=rest-registry-dataplane-2021-07-01#tagchangeableattributes).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct ContainerRegistryTagChangeableAttributes {
    pub delete_enabled: bool,
    pub list_enabled: bool,
    pub read_enabled: bool,
    pub write_enabled: bool,
}
cloud_terrastodon_registry::register_thing!(ContainerRegistryTagChangeableAttributes);
cloud_terrastodon_registry::register_arbitrary!(ContainerRegistryTagChangeableAttributes);
