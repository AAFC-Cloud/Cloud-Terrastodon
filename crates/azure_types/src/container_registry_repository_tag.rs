use crate::ContainerRegistryTagChangeableAttributes;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;

/// [TagAttributesBase, API 2021-07-01](https://learn.microsoft.com/en-us/rest/api/registry-dataplane/container-registry/get-tags?view=rest-registry-dataplane-2021-07-01#tagattributesbase).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct ContainerRegistryRepositoryTag {
    pub created_time: DateTime<Utc>,
    pub digest: String,
    pub last_update_time: DateTime<Utc>,
    pub name: String,
    pub changeable_attributes: ContainerRegistryTagChangeableAttributes,
    /// Omitted from the documented tag-list example; absence does not imply false.
    pub signed: Option<bool>,
}

cloud_terrastodon_registry::register_thing!(ContainerRegistryRepositoryTag);
cloud_terrastodon_registry::register_arbitrary!(ContainerRegistryRepositoryTag);

cloud_terrastodon_registry::register_arbitrary!(Vec<ContainerRegistryRepositoryTag>);
