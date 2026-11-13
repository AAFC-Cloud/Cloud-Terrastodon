use crate::MicrosoftGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::EntraUser;
use cloud_terrastodon_azure_types::PercentEncodeExt;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use eyre::Result;
use facet::Facet;
use std::sync::Arc;
use std::time::Duration;
use tracing::debug;

const USER_SEARCH_CACHE_DURATION: Duration = Duration::from_secs(60);

/// Search Graph v1.0 [users](https://learn.microsoft.com/graph/api/user-list?view=graph-rest-1.0) by a prefix of their identity or name.
#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Facet)]
pub struct EntraUserSearchRequest {
    pub search_term: String,
    pub authentication: AzureBearerAuthenticationSelection,
}

impl EntraUserSearchRequest {
    /// The invalidation root shared by all search terms in this tenant.
    pub(crate) fn tenant_cache_key(&self) -> Result<CacheKey> {
        Ok(CacheKey {
            scope: CacheScope::try_from_components([
                "ms",
                "graph",
                "GET",
                "users",
                "search",
                &self.authentication.tenant_id.to_string(),
            ])?,
            valid_for: USER_SEARCH_CACHE_DURATION,
        })
    }

    fn url(&self) -> String {
        let search_term = escape_odata_string(self.search_term.trim());
        let filter = format!(
            "startswith(displayName,'{search_term}') or startswith(givenName,'{search_term}') or startswith(surname,'{search_term}') or startswith(mail,'{search_term}') or startswith(userPrincipalName,'{search_term}')"
        );

        format!(
            "https://graph.microsoft.com/v1.0/users?$select={}&$filter={}",
            EntraUser::SELECT,
            filter.percent_encode()
        )
    }
}

impl InvalidatesOwnCacheKey for EntraUserSearchRequest {}

impl HasCacheKey for EntraUserSearchRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        let search_hash = blake3::hash(self.search_term.trim().as_bytes())
            .to_hex()
            .to_string();
        self.tenant_cache_key()?.join(search_hash)
    }
}

impl ExecutableRequest for EntraUserSearchRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<EntraUser>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let search_term = self.search_term.trim();
            if search_term.is_empty() {
                debug!(tenant_id = %self.authentication.tenant_id, "Skipping empty Entra user search");
                return Ok(Vec::new());
            }

            debug!(
                tenant_id = %self.authentication.tenant_id,
                search_term,
                "Searching Entra users"
            );
            let users: Vec<Arc<EntraUser>> =
                MicrosoftGraphHelper::new(self.url(), Some(self.cache_key()?), self.authentication)
                    .fetch_all(&executor)
                    .await?;
            debug!(tenant_id = %self.authentication.tenant_id, count = users.len(), "Found Entra users");
            Ok(users)
        })
    }
}

fn escape_odata_string(value: &str) -> String {
    value.replace('\'', "''")
}

cloud_terrastodon_registry::register_thing!(EntraUserSearchRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraUserSearchRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraUserSearchRequest => Vec<Arc<EntraUser>>, effects = [Read]);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_authentication::test_authentication;
    use cloud_terrastodon_command::CancellationToken;

    #[test]
    fn url_escapes_search_terms_as_odata_query_values() {
        CancellationToken::new()
            .with_scope(|_cancellation| {
                let authentication = test_authentication();
                let request = EntraUserSearchRequest {
                    search_term: "O'Neil & Smith".to_owned(),
                    authentication,
                };

                assert!(request.url().contains("O%27%27Neil%20%26%20Smith"));
                eyre::Ok(())
            })
            .expect("uncancelled test scope");
    }

    #[test]
    fn paginated_search_cache_expires() {
        CancellationToken::new()
            .with_scope(|_cancellation| {
                let authentication = test_authentication();
                let request = EntraUserSearchRequest {
                    search_term: "Smith".to_owned(),
                    authentication,
                };

                assert_eq!(request.cache_key()?.valid_for, Duration::from_secs(60));
                eyre::Ok(())
            })
            .expect("uncancelled test scope");
    }
}

cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraUserSearchRequest,
    name = "entra-user-search",
    cached
);
