use crate::AzureStorageAccountBlobContainer;
use facet::Facet;
use url::Url;

/// One ARM inventory page of the account's blob containers.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/list?view=rest-storagerp-2025-06-01#listcontaineritems).
/// The response's optional continuation URL is preserved, including when the
/// service chooses pagination without an explicit page size.
#[derive(Debug, Clone, PartialEq, Eq, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountBlobContainerListResponse {
    pub value: Vec<AzureStorageAccountBlobContainer>,
    /// Present only when another page exists; the ListDeletedContainers example omits it.
    pub next_link: Option<Url>,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountBlobContainerListResponse);
