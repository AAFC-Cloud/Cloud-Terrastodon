use crate::ContainerRegistryLoginServer;
use crate::ContainerRegistryRepositoryName;
use crate::ContainerRegistryRepositoryTag;

/// [TagList, API 2021-07-01](https://learn.microsoft.com/en-us/rest/api/registry-dataplane/container-registry/get-tags?view=rest-registry-dataplane-2021-07-01#taglist).
#[derive(Debug, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct ContainerRegistryTagListResponse {
    pub image_name: ContainerRegistryRepositoryName,
    pub registry: ContainerRegistryLoginServer,
    pub tags: Vec<ContainerRegistryRepositoryTag>,
    /// Opaque service continuation, which may be relative to the current page.
    pub link: Option<String>,
}
