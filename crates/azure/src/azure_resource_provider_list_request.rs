use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::AzureResourceProvider;
use cloud_terrastodon_azure_types::AzureResourceProviderListResponse;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use eyre::WrapErr;
use eyre::ensure;
use eyre::eyre;
use facet::Facet;
use http::Method;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use url::Url;

const PROVIDERS_API_VERSION: &str = "2021-04-01";

/// Cached provider metadata for one explicitly selected Azure tenant.
///
/// Uses [Providers - List At Tenant Scope, 2021-04-01](https://learn.microsoft.com/en-us/rest/api/resources/providers/list-at-tenant-scope?view=rest-resources-2021-04-01)
/// and follows its `nextLink` values. This public-cloud request does not include
/// the optional metadata/alias expansions. Use `with_invalidation(true)` or
/// [`CacheInvalidatable::invalidate`] to refresh all cached pages for this
/// tenant and authentication source.
///
/// Tenant metadata does not establish registration readiness in a particular
/// subscription. Subscription registration queries and changes require their
/// own subscription-scoped operations.
#[must_use = "This is a future request, you must .execute it"]
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureResourceProviderListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl<'a> Arbitrary<'a> for AzureResourceProviderListRequest {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            authentication: Arbitrary::arbitrary(u)?,
        })
    }
}

impl HasCacheKey for AzureResourceProviderListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(
            PathBuf::from("az/arm/tenant")
                .join(self.authentication.tenant_id.to_string())
                .join("auth-source")
                .join(self.authentication.auth_source.to_string())
                .join("providers/list"),
        )
    }
}

impl InvalidatesOwnCacheKey for AzureResourceProviderListRequest {}

impl ExecutableRequest for AzureResourceProviderListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<AzureResourceProvider>>;

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
    {
        Box::pin(async move {
            // Only REST pages have stored entries. Keeping them beneath a
            // transport-specific child avoids reusing a page key for a future
            // aggregate result and makes request invalidation cover every page.
            let page_cache = self
                .cache_key()?
                .join("rest/api-version")?
                .join(PROVIDERS_API_VERSION)?
                .join("pages")?;
            let mut url = Url::parse("https://management.azure.com/providers")
                .wrap_err("constructing the provider metadata endpoint")?;
            url.query_pairs_mut()
                .append_pair("api-version", PROVIDERS_API_VERSION);
            let mut next = Some(url);
            let mut visited = HashSet::new();
            let mut providers = Vec::new();
            while let Some(url) = next {
                executor.cancellation().bail_if_cancelled()?;
                Self::validate_page_url(&url)?;
                ensure!(
                    visited.insert(url.clone()),
                    "provider metadata pagination repeated a previously requested URL"
                );
                let page_key =
                    page_cache.join(blake3::hash(url.as_str().as_bytes()).to_hex().as_str())?;
                let page = RestRequest::from_method_and_url(Method::GET, url)?
                    .authentication(self.authentication.into())
                    .cache(page_key)
                    .receive_cached::<AzureResourceProviderListResponse>(
                        &executor,
                        CacheOperationOwner::of::<AzureResourceProviderListRequest>(),
                    )
                    .await?;
                executor.cancellation().bail_if_cancelled()?;
                providers.extend(
                    page.value
                        .as_ref()
                        .ok_or_else(|| {
                            eyre!("provider metadata response omitted its provider list")
                        })?
                        .iter()
                        .cloned(),
                );
                next = page.next_link.clone();
            }
            Ok(providers)
        })
    }
}

impl AzureResourceProviderListRequest {
    fn validate_page_url(url: &Url) -> Result<()> {
        // This is a library safety constraint, not a claim about every ARM
        // route: continuation requests may carry this tenant's credential only
        // to the same supported public-cloud provider collection.
        ensure!(
            url.scheme() == "https"
                && url.host_str() == Some("management.azure.com")
                && url.port_or_known_default() == Some(443)
                && url.username().is_empty()
                && url.password().is_none()
                && url.fragment().is_none()
                && url.path() == "/providers",
            "provider metadata continuation URL leaves the supported ARM provider collection"
        );
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(AzureResourceProviderListRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureResourceProviderListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureResourceProviderListRequest => Vec<Arc<AzureResourceProvider>>,
    effects = [Read]
);

#[cfg(test)]
mod tests {
    use super::AzureResourceProviderListRequest;
    use crate::test_authentication::test_authentication;
    use cloud_terrastodon_auth_types::AzureBearerAuthSource;
    use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
    use cloud_terrastodon_cache::HasCacheKey;
    use cloud_terrastodon_command::CancellationToken;
    use eyre::Result;

    #[test]
    fn continuation_cannot_redirect_credentials_away_from_the_provider_collection() -> Result<()> {
        assert!(
            AzureResourceProviderListRequest::validate_page_url(
                &"https://management.azure.com/providers?api-version=2021-04-01&%24skiptoken=next"
                    .parse()?
            )
            .is_ok()
        );
        for url in [
            "http://management.azure.com/providers",
            "https://other.example/providers",
            "https://management.azure.com:444/providers",
            "https://management.azure.com/subscriptions",
            "https://user@management.azure.com/providers",
            "https://management.azure.com/providers#fragment",
        ] {
            assert!(
                AzureResourceProviderListRequest::validate_page_url(&url.parse()?).is_err(),
                "{url}"
            );
        }
        Ok(())
    }

    #[tokio::test]
    async fn cache_scopes_separate_tenants_and_authentication_sources() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|_cancellation| async move {
                let first_tenant = "11111111-1111-1111-1111-111111111111".parse()?;
                let second_tenant = "22222222-2222-2222-2222-222222222222".parse()?;
                let first_cli = AzureBearerAuthenticationSelection {
                    tenant_id: first_tenant,
                    ..test_authentication()
                };
                let second_cli = AzureBearerAuthenticationSelection {
                    tenant_id: second_tenant,
                    ..test_authentication()
                };
                let first_browser = AzureBearerAuthenticationSelection {
                    tenant_id: first_tenant,
                    auth_source: AzureBearerAuthSource::Browser,
                    ..test_authentication()
                };
                let key = AzureResourceProviderListRequest {
                    authentication: first_cli,
                }
                .cache_key()?;
                assert_ne!(
                    key.scope,
                    AzureResourceProviderListRequest {
                        authentication: second_cli
                    }
                    .cache_key()?
                    .scope
                );
                assert_ne!(
                    key.scope,
                    AzureResourceProviderListRequest {
                        authentication: first_browser
                    }
                    .cache_key()?
                    .scope
                );
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureResourceProviderListRequest,
    name = "resource-provider-list",
    cached
);
