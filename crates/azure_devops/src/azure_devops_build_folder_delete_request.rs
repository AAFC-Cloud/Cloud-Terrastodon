use crate::azure_devops_rest::authenticate_azure_devops_request;
use crate::fetch_azure_devops_build_definitions;
use crate::fetch_azure_devops_build_folders;
use crate::fetch_azure_devops_builds;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildFolderPath;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_command::CacheInvalidatable;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::HasCacheKey;
use cloud_terrastodon_credentials::AzureDevOpsAuthContext;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use cloud_terrastodon_rest::RestRequest;
use eyre::Context;
use eyre::Result;
use eyre::ensure;
use reqwest::Method;
use reqwest::Url;
use std::borrow::Cow;
use std::future::Future;
use std::future::IntoFuture;
use std::path::PathBuf;
use std::pin::Pin;
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
#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsBuildFolderDeleteRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
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
                AzureDevOpsBuildFolderPath::try_new(r"\Synthetic").unwrap()
            } else {
                path
            },
        })
    }
}

impl HasCacheKey for AzureDevOpsBuildFolderDeleteRequest<'_> {
    fn cache_key(&self) -> CacheKey {
        let mut path = PathBuf::from("az/devops");
        if self.org_url.collection_name().is_some() {
            let (scheme, instance) = self
                .org_url
                .base_url()
                .split_once("://")
                .expect("validated organization URLs contain a scheme and instance");
            path.push("server");
            path.push(scheme);
            for component in instance.split('/') {
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
        CacheKey {
            path,
            // Keep inspectable artifacts, but execute every DELETE anew.
            valid_for: Duration::ZERO,
        }
    }
}

impl<'a> IntoFuture for AzureDevOpsBuildFolderDeleteRequest<'a> {
    type Output = Result<()>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
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
            let request = authenticate_azure_devops_request(
                RestRequest::new(Method::DELETE, url.as_str())?.cache(self.cache_key()),
                &self.auth_context,
            )?;
            let response = request.receive_raw().await?;
            // A successful DELETE may return HTTP 204 with no JSON body.
            ensure!(
                response.ok,
                "Deleting Azure DevOps build folder '{}' failed with status {}: {}",
                self.path,
                response.status,
                response.reason_phrase.as_deref().unwrap_or("Unknown error")
            );
            let folders = fetch_azure_devops_build_folders(
                &self.org_url,
                self.project.clone(),
                &self.auth_context,
                None,
            );
            let definitions = fetch_azure_devops_build_definitions(
                &self.org_url,
                self.project.clone(),
                &self.auth_context,
                None,
                None,
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
            );
            // A successful delete affects all three inventories. Attempt each
            // invalidation even if another fails, then preserve the source error.
            let (folders, definitions, builds) = tokio::join!(
                folders.invalidate(),
                definitions.invalidate(),
                builds.invalidate()
            );
            for result in [
                folders.wrap_err("Invalidating build folder listings failed"),
                definitions.wrap_err("Invalidating build definition listings failed"),
                builds.wrap_err("Invalidating build listings failed"),
            ] {
                result.wrap_err_with(|| {
                    format!(
                        "Build folder '{}' was deleted, but invalidating project '{}' build caches failed",
                        self.path, self.project
                    )
                })?;
            }
            Ok(())
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildFolderDeleteRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildFolderDeleteRequest<'static>);
cloud_terrastodon_registry::register_into_future!(
    AzureDevOpsBuildFolderDeleteRequest<'static> => (),
    effects = [Write]
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_folder_delete_artifacts_never_supply_cached_results() {
        let request = AzureDevOpsBuildFolderDeleteRequest {
            org_url: Cow::Owned("fixture".parse().unwrap()),
            project: "Synthetic project".parse().unwrap(),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
            path: AzureDevOpsBuildFolderPath::try_new(r"\Synthetic").unwrap(),
        };
        let key = request.cache_key();
        assert!(key.valid_for.is_zero());
        assert_eq!(
            key.path,
            PathBuf::from(
                "az/devops/org/fixture/project/synthetic%20project/build/definition/folder/delete/%5CSynthetic"
            )
        );
    }

    #[tokio::test]
    async fn build_folder_delete_rejects_root_before_authentication_or_network() {
        let error = AzureDevOpsBuildFolderDeleteRequest {
            org_url: Cow::Owned("fixture".parse().unwrap()),
            project: "Synthetic project".parse().unwrap(),
            // Missing auth also rejects locally if root protection regresses;
            // this test can never acquire credentials or execute HTTP.
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
            path: AzureDevOpsBuildFolderPath::root(),
        }
        .await
        .expect_err("root deletion must be rejected");
        assert!(error.to_string().contains("root build folder"));
    }
}
