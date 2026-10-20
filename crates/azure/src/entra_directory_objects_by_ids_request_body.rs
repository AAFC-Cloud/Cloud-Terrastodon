use cloud_terrastodon_azure_types::uuid::Uuid;
use facet::Facet;

/// The body of Graph v1.0 [`getByIds`](https://learn.microsoft.com/en-us/graph/api/directoryobject-getbyids?view=graph-rest-1.0#request-body).
///
/// Omitting `types` searches all directory-object types, preserving devices and
/// organizational contacts instead of filtering them out before resolution.
#[derive(Debug, Facet)]
pub(crate) struct EntraDirectoryObjectsByIdsRequestBody {
    pub ids: Vec<Uuid>,
}
