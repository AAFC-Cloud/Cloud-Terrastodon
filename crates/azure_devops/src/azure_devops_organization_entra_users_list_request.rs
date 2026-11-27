use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops_types::AzureDevOpsCollectionResponse;
use cloud_terrastodon_azure_devops_types::AzureDevOpsGraphUser;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use cloud_terrastodon_rest::MicrosoftContinuationToken;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use eyre::ensure;
use facet::Facet;
use reqwest::Method;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;

/// Lists Entra-backed identities visible through [Azure DevOps Graph Users List](https://learn.microsoft.com/en-us/rest/api/azure/devops/graph/users/list?view=azure-devops-rest-7.1).
///
/// The API uses the `aad` subject subtype and signals further pages through
/// the Microsoft continuation-token header.
#[derive(Debug, Clone, Arbitrary, Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsOrganizationEntraUsersListRequest {
    pub organization: AzureDevOpsOrganizationUrl,
    pub authentication: AzureDevOpsAuthSelection,
}

impl HasCacheKey for AzureDevOpsOrganizationEntraUsersListRequest {
    fn cache_key(&self) -> Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az/devops/org",
            &sanitize_windows_path_component(
                &self.organization.organization_name()?.to_ascii_lowercase(),
            ),
            "user/list/entra",
        ]))
    }
}

impl InvalidatesOwnCacheKey for AzureDevOpsOrganizationEntraUsersListRequest {}

impl ExecutableRequest for AzureDevOpsOrganizationEntraUsersListRequest {
    type Error = eyre::Report;
    type Output = Vec<Arc<AzureDevOpsGraphUser>>;

    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> Result<CloudTerrastodonExecutor> {
        self.organization.organization_name()?;
        executor.validate_authentication_target(
            &self.authentication.into(),
            &self.organization.api_url("dev.azure.com", "_apis", &[])?,
        )?;
        executor.with_authentication_lease(&self.authentication.into())
    }

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let pages = self.cache_key()?.join("pages")?;
            let owner = CacheOperationOwner::of::<Self>();
            let mut users = Vec::new();
            let mut continuation: Option<MicrosoftContinuationToken> = None;
            let mut seen = BTreeSet::new();
            let mut page_index = 0;
            loop {
                executor.cancellation().bail_if_cancelled()?;
                let mut query = vec![("api-version", "7.1-preview.1"), ("subjectTypes", "aad")];
                if let Some(token) = &continuation {
                    query.push(("continuationToken", token.as_str()));
                }
                let url = self.organization.api_url(
                    "vssps.dev.azure.com",
                    "_apis/graph/users",
                    &query,
                )?;
                let (response, next_continuation) = RestRequest::from_method_and_url(Method::GET, url)?
                    .cache(pages.join(page_index.to_string())?)
                    .authentication(self.authentication.into())
                    .receive_with_ms_continuation_token_cached::<AzureDevOpsCollectionResponse<Vec<Arc<AzureDevOpsGraphUser>>>>(&executor, owner)
                    .await?;
                users.extend(response.value.iter().cloned());
                let Some(token) = next_continuation else {
                    break;
                };
                ensure!(
                    seen.insert(token.clone()),
                    "Azure DevOps returned a repeated user continuation token"
                );
                continuation = Some(token);
                page_index += 1;
            }
            Ok(users)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsOrganizationEntraUsersListRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsOrganizationEntraUsersListRequest);
cloud_terrastodon_registry::register_thing!(Vec<Arc<AzureDevOpsGraphUser>>);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureDevOpsOrganizationEntraUsersListRequest => Vec<Arc<AzureDevOpsGraphUser>>,
    effects = [Read]
);
