use arbitrary::Arbitrary;
use cloud_terrastodon_auth::AzureDevOpsAuthContext;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProject;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use cloud_terrastodon_rest::RestRequest;
#[cfg(test)]
use eyre::Result;
use facet::Facet;
use reqwest::Method;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

#[derive(Debug, Clone, facet::Facet)]
// Facet's pinned derive cannot autodetect Clone on lifetime-generic types.
#[facet(traits(Clone))]
pub struct AzureDevOpsProjectsListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
}

#[track_caller]
pub fn fetch_all_azure_devops_projects<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    auth_context: &'a AzureDevOpsAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, AzureDevOpsProjectsListRequest<'a>> {
    executor.execute(AzureDevOpsProjectsListRequest {
        org_url: Cow::Borrowed(org_url),
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl<'a> Arbitrary<'a> for AzureDevOpsProjectsListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
        })
    }
}
impl<'a> HasCacheKey for AzureDevOpsProjectsListRequest<'a> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "devops",
            self.org_url.name(),
            "project",
            "list",
        ]))
    }
}

impl<'a> InvalidatesOwnCacheKey for AzureDevOpsProjectsListRequest<'a> {}

impl<'a> ExecutableRequest for AzureDevOpsProjectsListRequest<'a> {
    type Error = eyre::Report;
    type Output = Vec<Arc<AzureDevOpsProject>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!("Fetching Azure DevOps projects");
            #[derive(Facet)]
            #[facet(rename_all = "camelCase")]
            pub struct Response {
                value: Vec<Arc<AzureDevOpsProject>>,
            }

            let mut projects = Vec::new();
            let cache_key = self.cache_key()?;
            let mut page_index = 0;
            let query = [("api-version", "7.1")];
            let url = self
                .org_url
                .api_url("dev.azure.com", "_apis/projects", &query)?;
            // Attaching authentication only records request policy. RestRequest
            // resolves credentials after its cache lookup, so healthy cached pages
            // remain usable when the selected authentication source has expired.
            let request = RestRequest::from_method_and_url(Method::GET, url)?
                .cache(cache_key.join(page_index.to_string())?)
                .azure_devops_auth_context(self.auth_context.as_ref())?;
            let (mut response, mut continuation) = request
                .receive_with_ms_continuation_token_cached::<Response>(
                    &executor,
                    CacheOperationOwner::of::<AzureDevOpsProjectsListRequest<'static>>(),
                )
                .await?;
            projects.extend(response.value.iter().cloned());
            page_index += 1;

            while let Some(next_continuation) = continuation.take() {
                executor.cancellation().bail_if_cancelled()?;
                debug!("Fetching the next page of projects");
                let query = [
                    ("api-version", "7.1"),
                    ("continuationToken", next_continuation.as_str()),
                ];
                let url = self
                    .org_url
                    .api_url("dev.azure.com", "_apis/projects", &query)?;
                let request = RestRequest::from_method_and_url(Method::GET, url)?
                    .cache(cache_key.join(page_index.to_string())?)
                    .azure_devops_auth_context(self.auth_context.as_ref())?;
                (response, continuation) = request
                    .receive_with_ms_continuation_token_cached::<Response>(
                        &executor,
                        CacheOperationOwner::of::<AzureDevOpsProjectsListRequest<'static>>(),
                    )
                    .await?;
                projects.extend(response.value.iter().cloned());
                page_index += 1;
            }

            debug!("Found {} Azure DevOps projects", projects.len());
            Ok(projects)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsProjectsListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsProjectsListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureDevOpsProjectsListRequest<'static> => Vec<Arc<AzureDevOpsProject>>,
    effects = [Read]
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_default_organization_url;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_command::CancellationToken;
    use cloud_terrastodon_registry::RuntimeValue;
    use std::sync::LazyLock;

    #[test]
    fn reflected_project_list_clone_preserves_owned_and_borrowed_inputs() -> Result<()> {
        static ORG: LazyLock<AzureDevOpsOrganizationUrl> = LazyLock::new(|| {
            "offline-clone-example"
                .parse()
                .expect("valid synthetic organization")
        });
        static AUTH: AzureDevOpsAuthContext = AzureDevOpsAuthContext::None;

        for borrowed in [false, true] {
            CancellationToken::new().with_scope(|_scope| {
                let original = RuntimeValue::from_box(Box::new(AzureDevOpsProjectsListRequest {
                    org_url: if borrowed {
                        Cow::Borrowed(&*ORG)
                    } else {
                        Cow::Owned((*ORG).clone())
                    },
                    auth_context: if borrowed {
                        Cow::Borrowed(&AUTH)
                    } else {
                        Cow::Owned(AUTH.clone())
                    },
                }))?;
                // This is the same reflected clone used before a clone-mode
                // invocation. Do not invoke the request or resolve credentials.
                let cloned = original
                    .try_clone()?
                    .into_box::<AzureDevOpsProjectsListRequest<'static>>()?
                    .downcast::<AzureDevOpsProjectsListRequest<'static>>()
                    .expect("runtime cloning preserves the request type");
                assert_eq!(cloned.org_url.as_ref(), &*ORG);
                assert!(matches!(
                    cloned.auth_context.as_ref(),
                    AzureDevOpsAuthContext::None
                ));
                assert_eq!(matches!(cloned.org_url, Cow::Borrowed(_)), borrowed);
                assert_eq!(matches!(cloned.auth_context, Cow::Borrowed(_)), borrowed);
                Ok::<_, eyre::Report>(())
            })?;
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_fetch_all_azure_devops_projects() -> Result<()> {
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
                let projects = fetch_all_azure_devops_projects(
                    &org_url,
                    &azure_devops_auth_context,
                    &executor,
                )
                .await?;
                assert!(!projects.is_empty());
                assert!(projects.iter().all(|project| !project.name.is_empty()));
                Ok(())
            })
            .await
    }
}
