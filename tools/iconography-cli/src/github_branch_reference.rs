use crate::GitHubObjectReference;
use url::Url;

/// A GitHub Git reference returned by the public reference endpoint.
///
/// See [GitHub REST: Get a reference](https://docs.github.com/en/rest/git/refs#get-a-reference).
#[derive(Debug, facet::Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct GitHubBranchReference {
    #[facet(rename = "ref")]
    pub reference: String,
    pub node_id: String,
    pub url: Url,
    pub object: GitHubObjectReference,
}
