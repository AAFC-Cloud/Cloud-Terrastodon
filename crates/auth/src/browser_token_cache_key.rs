use cloud_terrastodon_auth_types::AzureCredentialResource;

/// One resource and delegated permission set within an admitted browser session.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct BrowserTokenCacheKey {
    pub(crate) resource: AzureCredentialResource,
    pub(crate) scopes: String,
}
