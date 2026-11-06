use crate::fetch_azure_devops_build_definitions;
use crate::fetch_azure_devops_build_folders;
use crate::fetch_azure_devops_builds;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth::AzureDevOpsAuthContext;
use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildFolderPath;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use cloud_terrastodon_rest::RestRequest;
use eyre::Context;
use eyre::ensure;
use reqwest::Method;
use reqwest::Url;
use std::borrow::Cow;
use std::path::PathBuf;
use std::time::Duration;

/// Delete a nonroot build folder using its exact server path.
///
/// This operation also deletes contained definitions and builds. Use
/// [`crate::AzureDevOpsBuildFolderPruneRequest`] to inspect and recheck emptiness
/// before deletion. The service offers no atomic empty-only delete condition.
/// Successful deletion invalidates build, definition, and folder list caches
/// for the selected project, including every filter variant and page.
/// A zero-validity cache records DELETE artifacts without reusing results.
///
/// See Microsoft's [Folders Delete API](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/folders/delete?view=azure-devops-rest-7.1)
/// (7.1-preview.2).
#[must_use = "This is a future request, you must .execute it"]
#[derive(Debug, Clone, facet::Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsBuildFolderDeleteRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    /// Project ID or name.
    pub project: AzureDevOpsProjectArgument<'a>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
    /// Exact folder spelling returned by the service, without CLI normalization.
    pub path: AzureDevOpsBuildFolderPath,
}

impl<'a> Arbitrary<'a> for AzureDevOpsBuildFolderDeleteRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let path = AzureDevOpsBuildFolderPath::arbitrary(u)?;
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
            path: if path.is_root() {
                AzureDevOpsBuildFolderPath::try_new(r"\Synthetic")
                    .map_err(|_| arbitrary::Error::IncorrectFormat)?
            } else {
                path
            },
        })
    }
}

impl HasCacheKey for AzureDevOpsBuildFolderDeleteRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        let mut path = PathBuf::from("az/devops");
        if self.org_url.collection_name().is_some() {
            path.push("server");
            path.push(self.org_url.instance_scheme());
            for component in self.org_url.instance_authority_and_path().split('/') {
                path.push(sanitize_windows_path_component(component));
            }
            path.push("org");
            path.push(sanitize_windows_path_component(self.org_url.name()));
        } else {
            path.push("org");
            path.push(sanitize_windows_path_component(
                &self.org_url.name().to_ascii_lowercase(),
            ));
        }
        let project = match &self.project {
            AzureDevOpsProjectArgument::Id(id) => id.to_string(),
            AzureDevOpsProjectArgument::Name(name) => {
                let name = name.to_string().to_ascii_lowercase();
                let component = sanitize_windows_path_component(&name);
                if name
                    .parse::<cloud_terrastodon_azure_devops_types::AzureDevOpsProjectId>()
                    .is_ok()
                {
                    format!("name={component}")
                } else {
                    component
                }
            }
        };
        path.push("project");
        path.push(project);
        path.extend(["build", "definition", "folder", "delete"]);
        path.push(sanitize_windows_path_component(self.path.as_str()));
        let mut key = CacheKey::from_relative_path(path)?;
        // Keep inspectable artifacts, but execute every DELETE anew.
        key.valid_for = Duration::ZERO;
        Ok(key)
    }
}

impl<'a> ExecutableRequest for AzureDevOpsBuildFolderDeleteRequest<'a> {
    type Error = eyre::Report;
    type Output = ();

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            ensure!(
                !self.path.is_root(),
                "The root build folder cannot be deleted"
            );
            let mut url = Url::parse(&self.org_url.to_string())?;
            url.path_segments_mut()
                .map_err(|()| {
                    eyre::eyre!(
                        "Azure DevOps organization URL does not support hierarchical path segments"
                    )
                })?
                .pop_if_empty()
                .push(&self.project.to_string())
                .extend(["_apis", "build", "folders"]);
            url.query_pairs_mut()
                .append_pair("api-version", "7.1-preview.2")
                .append_pair("path", self.path.as_str());
            executor.deferred(
                    "finishing folder deletion and invalidating affected build caches",
                    |executor| async move {
                let request =
                    RestRequest::from_method_and_url(Method::DELETE, url)?
                        .cache(self.cache_key()?)
                        .azure_devops_auth_context(self.auth_context.as_ref())?;
                let result = request.receive_raw(&executor).await.and_then(|response| {
                    // A successful DELETE may return HTTP 204 with no JSON body.
                    ensure!(
                        response.ok,
                        "Deleting Azure DevOps build folder '{}' failed with status {}: {}",
                        self.path,
                        response.status.as_u16(),
                        response.reason_phrase.as_deref().unwrap_or("Unknown error")
                    );
                    Ok(())
                });
                let folders = fetch_azure_devops_build_folders(
                    &self.org_url,
                    self.project.clone(),
                    &self.auth_context,
                    None,
                    &executor,
                );
                let definitions = fetch_azure_devops_build_definitions(
                    &self.org_url,
                    self.project.clone(),
                    &self.auth_context,
                    None,
                    None,
                    &executor,
                );
                let builds = fetch_azure_devops_builds(
                    &self.org_url,
                    self.project.clone(),
                    &self.auth_context,
                    Vec::new(),
                    None,
                    None,
                    None,
                    None,
                    &executor,
                );
                // Transport or artifact completion can fail after DELETE took effect.
                // Attempt each affected invalidation before returning either failure.
                let (folders, definitions, builds) = tokio::join!(
                    folders.invalidate(),
                    definitions.invalidate(),
                    builds.invalidate()
                );
                let invalidation = [
                    folders.wrap_err("Invalidating build folder listings failed"),
                    definitions.wrap_err("Invalidating build definition listings failed"),
                    builds.wrap_err("Invalidating build listings failed"),
                ].into_iter().try_for_each(|result| {
                    result.wrap_err_with(|| {
                        format!(
                            "Invalidating project '{}' build caches after attempting to delete folder '{}' failed",
                            self.project, self.path
                        )
                    })
                });
                match (result, invalidation) {
                    (Ok(()), Ok(())) => Ok(()),
                    (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
                    (Err(error), Err(invalidation)) => Err(error.wrap_err(format!(
                        "affected build cache invalidation also failed: {invalidation:#}"
                    ))),
                }
                    },
                )
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildFolderDeleteRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildFolderDeleteRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureDevOpsBuildFolderDeleteRequest<'static> => (),
    effects = [Write]
);

impl InvalidatesOwnCacheKey for AzureDevOpsBuildFolderDeleteRequest<'_> {}
#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_command::CancellationToken;

    #[test]
    fn build_folder_delete_artifacts_never_supply_cached_results() {
        let request = AzureDevOpsBuildFolderDeleteRequest {
            org_url: Cow::Owned("https://dev.azure.com/fixture".parse().unwrap()),
            project: "Synthetic project".parse().unwrap(),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
            path: AzureDevOpsBuildFolderPath::try_new(r"\Synthetic").unwrap(),
        };
        let key = request.cache_key().unwrap();
        assert!(key.valid_for.is_zero());
        assert_eq!(
            key.scope.components().iter().collect::<PathBuf>(),
            PathBuf::from(
                "az/devops/org/fixture/project/synthetic%20project/build/definition/folder/delete/%5CSynthetic"
            )
        );
    }

    #[tokio::test]
    async fn build_folder_delete_rejects_root_before_authentication_or_network() {
        CancellationToken::new()
            .with_async_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AzureDevOpsAuthContext::None,
                    &cancellation,
                )?;
                let error = AzureDevOpsBuildFolderDeleteRequest {
                    org_url: Cow::Owned("https://dev.azure.com/fixture".parse().unwrap()),
                    project: "Synthetic project".parse().unwrap(),
                    // Missing auth also rejects locally if root protection regresses;
                    // this test can never acquire credentials or execute HTTP.
                    auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
                    path: AzureDevOpsBuildFolderPath::root(),
                }
                .execute(&executor)
                .await
                .expect_err("root deletion must be rejected");
                assert!(error.to_string().contains("root build folder"));
                Ok::<_, eyre::Report>(())
            })
            .await
            .unwrap();
    }
}
