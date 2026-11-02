use url::Url;

/// The Git object selected by a GitHub reference.
///
/// See the `object` property in
/// [GitHub REST: Get a reference](https://docs.github.com/en/rest/git/refs#get-a-reference).
#[derive(Debug, facet::Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct GitHubObjectReference {
    pub sha: String,
    #[facet(rename = "type")]
    pub kind: String,
    pub url: Url,
}
