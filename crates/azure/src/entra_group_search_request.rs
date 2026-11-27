use crate::MicrosoftGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::EntraGroup;
use cloud_terrastodon_azure_types::EntraGroupId;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_rest::RequestHeaders;
use facet::Facet;
use std::sync::Arc;
use std::time::Duration;
use url::Url;

/// Find Microsoft Graph v1.0 [groups](https://learn.microsoft.com/graph/api/group-list?view=graph-rest-1.0)
/// by exact object ID or [tokenized display-name search](https://learn.microsoft.com/graph/search-query-parameter#use-search-on-directory-object-collections).
#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Facet)]
pub struct EntraGroupSearchRequest {
    pub search_term: String,
    pub authentication: AzureBearerAuthenticationSelection,
}

impl InvalidatesOwnCacheKey for EntraGroupSearchRequest {}

impl HasCacheKey for EntraGroupSearchRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        let tenant = self.authentication.tenant_id.to_string();
        let search = blake3::hash(self.search_term.trim().as_bytes()).to_hex();
        Ok(CacheKey {
            scope: CacheScope::try_from_components([
                "ms",
                "graph",
                "GET",
                "groups",
                "search",
                &tenant,
                search.as_str(),
            ])?,
            valid_for: Duration::from_secs(60),
        })
    }
}

impl ExecutableRequest for EntraGroupSearchRequest {
    type Error = eyre::Report;
    type Output = Vec<Arc<EntraGroup>>;

    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor> {
        executor.with_authentication_lease(&self.authentication.into())
    }

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let search_term = self.search_term.trim();
            if search_term.is_empty() {
                return eyre::Ok(Vec::new());
            }
            let cache_key = Some(self.cache_key()?);
            let mut url = Url::parse("https://graph.microsoft.com/v1.0/groups")?;
            let graph = match search_term.parse::<EntraGroupId>() {
                Ok(id) => {
                    url.query_pairs_mut()
                        .append_pair("$filter", &format!("id eq '{id}'"));
                    MicrosoftGraphHelper::new(url, cache_key, self.authentication)
                }
                Err(_) => {
                    let escaped = search_term.replace('\\', "\\\\").replace('"', "\\\"");
                    url.query_pairs_mut()
                        .append_pair("$search", &format!(r#""displayName:{escaped}""#));
                    // Microsoft Graph's v1.0 search needs this preference to handle encoded ampersands.
                    // https://learn.microsoft.com/graph/known-issues#search-for-directory-objects-fails-for-encoded-ampersand--character
                    MicrosoftGraphHelper::new(url, cache_key, self.authentication)
                        .with_consistency_level_eventual()?
                        .add_headers(RequestHeaders::from_header("Prefer", "legacySearch=false")?)
                }
            };
            graph.fetch_all(&executor).await
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraGroupSearchRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraGroupSearchRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraGroupSearchRequest => Vec<Arc<EntraGroup>>, effects = [Read]);
