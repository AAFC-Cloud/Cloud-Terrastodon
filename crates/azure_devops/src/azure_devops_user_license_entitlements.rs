use arbitrary::Arbitrary;
use cloud_terrastodon_auth::AzureDevOpsAuthContext;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsUserLicenseEntitlement;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use cloud_terrastodon_rest::MicrosoftContinuationToken;
use cloud_terrastodon_rest::RestRequest;
use facet::Facet;
use reqwest::Method;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

#[derive(Debug, Clone, facet::Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsUserLicenseEntitlementListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
}

#[track_caller]
pub fn fetch_azure_devops_user_license_entitlements<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    auth_context: &'a AzureDevOpsAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, AzureDevOpsUserLicenseEntitlementListRequest<'a>> {
    executor.execute(AzureDevOpsUserLicenseEntitlementListRequest {
        org_url: Cow::Borrowed(org_url),
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl<'a> Arbitrary<'a> for AzureDevOpsUserLicenseEntitlementListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
        })
    }
}
impl<'a> HasCacheKey for AzureDevOpsUserLicenseEntitlementListRequest<'a> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "devops",
            self.org_url.name(),
            "license",
            "entitlement",
            "list",
        ]))
    }
}

impl<'a> InvalidatesOwnCacheKey for AzureDevOpsUserLicenseEntitlementListRequest<'a> {}

impl<'a> ExecutableRequest for AzureDevOpsUserLicenseEntitlementListRequest<'a> {
    type Error = eyre::Report;
    type Output = Vec<Arc<AzureDevOpsUserLicenseEntitlement>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!("Fetching Azure DevOps user entitlements");
            #[derive(Facet)]
            #[facet(rename_all = "camelCase")]
            struct InvokeResponse {
                count: u32,
                value: Vec<Arc<AzureDevOpsUserLicenseEntitlement>>,
            }

            let mut entitlements = Vec::new();
            let mut continuation: Option<MicrosoftContinuationToken> = None;
            let mut count = 0;
            let cache_key = self.cache_key()?;
            let mut page_index = 0;
            loop {
                let mut query = vec![("api-version", "7.1-preview.3")];
                if let Some(token) = continuation.as_ref() {
                    query.push(("continuationToken", token.as_str()));
                }
                let url = self.org_url.api_url(
                    "vsaex.dev.azure.com",
                    "_apis/userentitlements",
                    &query,
                )?;
                let request = RestRequest::from_method_and_url(Method::GET, url)?
                    .cache(cache_key.join(page_index.to_string())?)
                    .azure_devops_auth_context(self.auth_context.as_ref())?;
                let (response, next_continuation) = request
                    .receive_with_ms_continuation_token_cached::<InvokeResponse>(
                        &executor,
                        CacheOperationOwner::of::<
                            AzureDevOpsUserLicenseEntitlementListRequest<'static>,
                        >(),
                    )
                    .await?;
                count += response.count;
                entitlements.extend(response.value.iter().cloned());
                continuation = next_continuation;
                page_index += 1;
                if continuation.is_none() {
                    break;
                }
            }

            debug!("Found {count} Azure DevOps user entitlements");
            Ok(entitlements)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsUserLicenseEntitlementListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(
    AzureDevOpsUserLicenseEntitlementListRequest<'static>
);
cloud_terrastodon_registry_execution::register_executable_request!(AzureDevOpsUserLicenseEntitlementListRequest<'static> => Vec<Arc<AzureDevOpsUserLicenseEntitlement>>, effects = [Read]);

#[cfg(test)]
mod test {
    use super::*;
    use crate::get_default_organization_url;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .with_async_scope(|cancellation| async move {
                let auth_context = AuthContext::explicit_azure_cli();
                let azure_devops_auth_context = AzureDevOpsAuthContext::for_tenant(
                    &auth_context,
                    crate::test_tenant_id::test_tenant_id()?,
                )?;
                let executor =
                    crate::test_executor::test_executor(&azure_devops_auth_context, &cancellation)?;
                let org_url = get_default_organization_url(&cancellation).await?;
                let entitlements = fetch_azure_devops_user_license_entitlements(
                    &org_url,
                    &azure_devops_auth_context,
                    &executor,
                )
                .await?;
                assert!(
                    !entitlements.is_empty(),
                    "Expected at least one Azure DevOps user entitlement"
                );
                assert!(
                    entitlements.iter().all(|entitlement| {
                        !entitlement.user.display_name.is_empty()
                            && !entitlement.user.unique_name.is_empty()
                    }),
                    "Expected sampled Azure DevOps user entitlements to include user identity data"
                );

                Ok(())
            })
            .await
    }
}
