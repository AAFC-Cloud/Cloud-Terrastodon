use arbitrary::Arbitrary;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::AzureResourceProvider;
use cloud_terrastodon_azure_types::AzureResourceProviderListResponse;
use cloud_terrastodon_command::CacheInvalidatable;
use cloud_terrastodon_command::CacheInvalidatableIntoFuture;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::HasCacheKey;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use eyre::WrapErr;
use eyre::ensure;
use eyre::eyre;
use facet::Facet;
use http::Method;
use std::borrow::Cow;
use std::collections::HashSet;
use std::future::Future;
use std::future::IntoFuture;
use std::path::PathBuf;
use std::pin::Pin;
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
#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct ResourceProviderListRequest<'a> {
    #[facet(skip_serializing)]
    pub cancellation: CancellationScope,
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

/// List provider metadata for the selected tenant, including all response pages.
///
/// See [Providers - List At Tenant Scope, 2021-04-01](https://learn.microsoft.com/en-us/rest/api/resources/providers/list-at-tenant-scope?view=rest-resources-2021-04-01).
pub fn fetch_all_resource_providers<'a>(
    auth_context: &'a AzureTenantAuthContext,
    cancellation: &CancellationScope,
) -> ResourceProviderListRequest<'a> {
    ResourceProviderListRequest {
        cancellation: cancellation.clone(),
        auth_context: Cow::Borrowed(auth_context),
    }
}

impl<'a> Arbitrary<'a> for ResourceProviderListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            cancellation: CancellationScope::arbitrary(u)?,
            auth_context: Cow::Owned(AzureTenantAuthContext::arbitrary(u)?),
        })
    }
}

impl HasCacheKey for ResourceProviderListRequest<'_> {
    fn cache_key(&self) -> CacheKey {
        CacheKey::new(
            PathBuf::from("az/arm/tenant")
                .join(self.auth_context.tenant_id.to_string())
                .join("auth-source")
                .join(
                    self.auth_context
                        .auth_context
                        .source()
                        .map(|source| source.to_string())
                        .unwrap_or_else(|| "placeholder".to_owned()),
                )
                .join("providers/list"),
        )
    }
}

impl<'a> CacheInvalidatableIntoFuture for ResourceProviderListRequest<'a> {
    type WithInvalidation = <Self as IntoFuture>::IntoFuture;

    fn with_invalidation(self, invalidate_cache: bool) -> Self::WithInvalidation {
        Box::pin(async move {
            if invalidate_cache {
                self.cancellation
                    .deferred(
                        "finishing filesystem and memory cache invalidation",
                        |_invalidation| async { self.invalidate().await },
                    )
                    .await?;
            }
            self.into_future().await
        })
    }
}

impl<'a> IntoFuture for ResourceProviderListRequest<'a> {
    type Output = Result<Vec<AzureResourceProvider>>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            // Only REST pages have stored entries. Keeping them beneath a
            // transport-specific child avoids reusing a page key for a future
            // aggregate result and makes request invalidation cover every page.
            let page_cache = self
                .cache_key()
                .join("rest/api-version")
                .join(PROVIDERS_API_VERSION)
                .join("pages");
            let mut url = Url::parse("https://management.azure.com/providers")
                .wrap_err("constructing the provider metadata endpoint")?;
            url.query_pairs_mut()
                .append_pair("api-version", PROVIDERS_API_VERSION);
            let mut next = Some(url);
            let mut visited = HashSet::new();
            let mut providers = Vec::new();
            while let Some(url) = next {
                self.cancellation.bail_if_cancelled()?;
                Self::validate_page_url(&url)?;
                ensure!(
                    visited.insert(url.clone()),
                    "provider metadata pagination repeated a previously requested URL"
                );
                let page_key =
                    page_cache.join(blake3::hash(url.as_str().as_bytes()).to_hex().as_str());
                let page = RestRequest::from_method_and_url(Method::GET, url, &self.cancellation)?
                    .azure_auth_context(&self.auth_context)
                    .cache(page_key)
                    .receive::<AzureResourceProviderListResponse>()
                    .await?;
                self.cancellation.bail_if_cancelled()?;
                providers.extend(page.value.ok_or_else(|| {
                    eyre!("provider metadata response omitted its provider list")
                })?);
                next = page.next_link;
            }
            Ok(providers)
        })
    }
}

impl ResourceProviderListRequest<'_> {
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

cloud_terrastodon_registry::register_thing!(ResourceProviderListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(ResourceProviderListRequest<'static>);
cloud_terrastodon_registry_execution::register_into_future!(
    ResourceProviderListRequest<'static> => Vec<AzureResourceProvider>,
    effects = [Read]
);

#[cfg(test)]
mod tests {
    use super::ResourceProviderListRequest;
    use super::fetch_all_resource_providers;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_auth_types::AuthSource;
    use cloud_terrastodon_command::CancellationToken;
    use cloud_terrastodon_command::HasCacheKey;
    use eyre::Result;

    #[test]
    fn continuation_cannot_redirect_credentials_away_from_the_provider_collection() -> Result<()> {
        assert!(
            ResourceProviderListRequest::validate_page_url(
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
                ResourceProviderListRequest::validate_page_url(&url.parse()?).is_err(),
                "{url}"
            );
        }
        Ok(())
    }

    #[tokio::test]
    async fn cache_scopes_separate_tenants_and_authentication_sources() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let first_tenant = "11111111-1111-1111-1111-111111111111".parse()?;
                let second_tenant = "22222222-2222-2222-2222-222222222222".parse()?;
                let first_cli = AuthContext::explicit(AuthSource::AzureCli)
                    .bind_to_azure_tenant(first_tenant)?;
                let second_cli = AuthContext::explicit(AuthSource::AzureCli)
                    .bind_to_azure_tenant(second_tenant)?;
                let first_browser = AuthContext::explicit(AuthSource::Browser)
                    .bind_to_azure_tenant(first_tenant)?;
                let key = fetch_all_resource_providers(&first_cli, &cancellation).cache_key();
                assert_ne!(
                    key.path,
                    fetch_all_resource_providers(&second_cli, &cancellation)
                        .cache_key()
                        .path
                );
                assert_ne!(
                    key.path,
                    fetch_all_resource_providers(&first_browser, &cancellation)
                        .cache_key()
                        .path
                );
                Ok(())
            })
            .await
    }
}
