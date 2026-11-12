use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops_types::AzureDevOpsCollectionResponse;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProject;
use cloud_terrastodon_cache::ArtifactObservation;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::ExecutorKind;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_cache::NonEmptyString;
use cloud_terrastodon_cache::WorkOutput;
use cloud_terrastodon_cache::WorkReuse;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use cloud_terrastodon_rest::MicrosoftContinuationToken;
use cloud_terrastodon_rest::RestRequest;
#[cfg(test)]
use eyre::Result;
use facet::Facet;
use reqwest::Method;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

/// The complete ordered project inventory for an explicit organization and
/// credential selection. The executor owns runtime credentials and shared results.
///
/// See [Projects List (7.1)](https://learn.microsoft.com/en-us/rest/api/azure/devops/core/projects/list?view=azure-devops-rest-7.1).
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsProjectsListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub auth_selection: AzureDevOpsAuthSelection,
}

impl<'a> Arbitrary<'a> for AzureDevOpsProjectsListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            auth_selection: AzureDevOpsAuthSelection::arbitrary(u)?,
        })
    }
}

impl HasCacheKey for AzureDevOpsProjectsListRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        let mut path = PathBuf::from("az/devops");
        if self.org_url.collection_name().is_some() {
            path.push("server");
            path.push(self.org_url.instance_scheme());
            for segment in self.org_url.instance_authority_and_path().split('/') {
                path.push(sanitize_windows_path_component(segment));
            }
            path.push("org");
            path.push(sanitize_windows_path_component(self.org_url.name()));
        } else {
            path.push("org");
            path.push(sanitize_windows_path_component(
                &self.org_url.name().to_ascii_lowercase(),
            ));
        }
        CacheKey::from_relative_path(path.join("project/list"))
    }
}

impl InvalidatesOwnCacheKey for AzureDevOpsProjectsListRequest<'_> {}

impl ExecutableRequest for AzureDevOpsProjectsListRequest<'_> {
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
    type Output = Arc<Vec<Arc<AzureDevOpsProject>>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let cache_key = self.cache_key()?;
            let selection = AuthenticationSelection::from(self.auth_selection);
            let fingerprint = NonEmptyString::new(
                blake3::hash(&facet_json::to_vec(&self)?)
                    .to_hex()
                    .to_string(),
            )?;
            executor
                .cached_work(
                    ExecutorKind::InProcess,
                    cache_key.clone(),
                    CacheOperationOwner::of::<AzureDevOpsProjectsListRequest<'static>>(),
                    &selection,
                    fingerprint,
                    WorkReuse::ReusableRead,
                    move |executor| async move {
                        debug!("Fetching Azure DevOps projects");
                        let observation = ArtifactObservation::now();
                        let mut projects = Vec::new();
                        let mut page_index = 0;
                        let mut continuation: Option<MicrosoftContinuationToken> = None;
                        loop {
                            let mut query = vec![("api-version", "7.1")];
                            if let Some(token) = &continuation {
                                query.push(("continuationToken", token.as_str()));
                            }
                            let url = self.org_url.api_url("dev.azure.com", "_apis/projects", &query)?;
                            // The initializer executor records each page as a
                            // dependency. Its age/invalidation also expires this
                            // memory-only aggregate; no aggregate is restored from disk.
                            let (response, next) = RestRequest::from_method_and_url(Method::GET, url)?
                                .cache(cache_key.join(page_index.to_string())?)
                                .authentication(selection)
                                .receive_with_ms_continuation_token_cached::<AzureDevOpsCollectionResponse<Vec<Arc<AzureDevOpsProject>>>>(
                                    &executor,
                                    CacheOperationOwner::of::<AzureDevOpsProjectsListRequest<'static>>(),
                                )
                                .await?;
                            projects.extend(response.value.iter().cloned());
                            executor.cancellation().bail_if_cancelled()?;
                            continuation = next;
                            if continuation.is_none() {
                                break;
                            }
                            page_index += 1;
                        }
                        debug!("Found {} Azure DevOps projects", projects.len());
                        Ok(WorkOutput::new(projects, observation))
                    },
                )?
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsProjectsListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsProjectsListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureDevOpsProjectsListRequest<'static> => Arc<Vec<Arc<AzureDevOpsProject>>>,
    effects = [Read]
);

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_registry::RuntimeValue;
    use std::sync::LazyLock;

    #[test]
    fn reflected_project_list_clone_preserves_request_data() -> Result<()> {
        static ORG: LazyLock<AzureDevOpsOrganizationUrl> = LazyLock::new(|| {
            "offline-clone-example"
                .parse()
                .expect("valid synthetic organization")
        });
        let auth_selection = crate::test_executor::authentication_selection();
        for borrowed in [false, true] {
            let original = RuntimeValue::from_box(Box::new(AzureDevOpsProjectsListRequest {
                org_url: if borrowed {
                    Cow::Borrowed(&*ORG)
                } else {
                    Cow::Owned((*ORG).clone())
                },
                auth_selection,
            }))?;
            // This is the reflected clone used before a clone-mode invocation.
            // Do not invoke the request or resolve credentials.
            let cloned = original
                .try_clone()?
                .into_box::<AzureDevOpsProjectsListRequest<'static>>()?
                .downcast::<AzureDevOpsProjectsListRequest<'static>>()
                .expect("runtime cloning preserves the request type");
            assert_eq!(cloned.org_url.as_ref(), &*ORG);
            assert_eq!(matches!(cloned.org_url, Cow::Borrowed(_)), borrowed);
            assert_eq!(cloned.auth_selection, auth_selection);
        }
        Ok(())
    }
}

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureDevOpsProjectsListRequest<'static>,
    name = "azure-devops-projects-list",
    cached
);
