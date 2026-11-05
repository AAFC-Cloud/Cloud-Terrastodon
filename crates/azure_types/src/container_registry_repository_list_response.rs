use crate::ContainerRegistryRepositoryName;

/// [Repositories, API 2021-07-01](https://learn.microsoft.com/en-us/rest/api/registry-dataplane/container-registry/get-repositories?view=rest-registry-dataplane-2021-07-01#repositories).
#[derive(Debug, facet::Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct ContainerRegistryRepositoryListResponse {
    pub repositories: Vec<ContainerRegistryRepositoryName>,
    /// Opaque service continuation, which may be relative to the current page.
    pub link: Option<String>,
}
