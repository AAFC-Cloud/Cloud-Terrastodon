use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsUserLicenseEntitlement;
use cloud_terrastodon_azure_devops_types::AzureDevOpsUserLicenseEntitlementPage;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_rest::MicrosoftContinuationToken;
use cloud_terrastodon_rest::RestRequest;
use eyre::WrapErr;
use eyre::ensure;
use facet::Facet;
use reqwest::Method;
use std::borrow::Cow;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

/// List user licenses with the [7.1 Search User Entitlements API](https://learn.microsoft.com/en-us/rest/api/azure/devops/memberentitlementmanagement/user-entitlements/search-user-entitlements?view=azure-devops-rest-7.1).
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsUserLicenseEntitlementListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub auth_selection: AzureDevOpsAuthSelection,
}

impl<'a> Arbitrary<'a> for AzureDevOpsUserLicenseEntitlementListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            auth_selection: AzureDevOpsAuthSelection::arbitrary(u)?,
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
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> Result<CloudTerrastodonExecutor, Self::Error> {
        executor.validate_authentication_target(
            &self.auth_selection.into(),
            &self.org_url.api_url("dev.azure.com", "_apis", &[])?,
        )?;
        executor.with_authentication_lease(&self.auth_selection.into())
    }

    type Error = eyre::Report;
    type Output = Vec<Arc<AzureDevOpsUserLicenseEntitlement>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!("Fetching Azure DevOps user entitlements");
            let mut entitlements = Vec::new();
            let mut continuation: Option<MicrosoftContinuationToken> = None;
            let mut seen = HashSet::new();
            let cache_key = self.cache_key()?;
            let mut page_index = 0;
            loop {
                let mut query = vec![("api-version", "7.1"), ("select", "license")];
                if let Some(token) = continuation.as_deref() {
                    query.push(("continuationToken", token));
                }
                let url = self.org_url.api_url(
                    "vsaex.dev.azure.com",
                    "_apis/userentitlements",
                    &query,
                )?;
                let request = RestRequest::from_method_and_url(Method::GET, url)?
                    .cache(cache_key.join(page_index.to_string())?)
                    .authentication(self.auth_selection.into());
                let page = request
                    .receive_cached::<AzureDevOpsUserLicenseEntitlementPage>(
                        &executor,
                        CacheOperationOwner::of::<
                            AzureDevOpsUserLicenseEntitlementListRequest<'static>,
                        >(),
                    )
                    .await?;
                executor.cancellation().bail_if_cancelled()?;
                entitlements.extend(page.items.iter().cloned());
                // The response body can use an empty token to end pagination.
                let Some(token) = page
                    .continuation_token
                    .as_deref()
                    .filter(|token| !token.is_empty())
                else {
                    break;
                };
                let token = MicrosoftContinuationToken::try_from(token)
                    .wrap_err("validating the Azure DevOps user entitlement continuation token")?;
                ensure!(
                    seen.insert(token.clone()),
                    "Azure DevOps returned a repeated user entitlement continuation token"
                );
                continuation = Some(token);
                page_index += 1;
            }

            debug!(
                count = entitlements.len(),
                "Found Azure DevOps user entitlements"
            );
            Ok(entitlements)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsUserLicenseEntitlementListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(
    AzureDevOpsUserLicenseEntitlementListRequest<'static>
);
cloud_terrastodon_registry_execution::register_executable_request!(AzureDevOpsUserLicenseEntitlementListRequest<'static> => Vec<Arc<AzureDevOpsUserLicenseEntitlement>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureDevOpsUserLicenseEntitlementListRequest<'static>,
    name = "azure-devops-user-license-entitlement-list",
    cached
);
