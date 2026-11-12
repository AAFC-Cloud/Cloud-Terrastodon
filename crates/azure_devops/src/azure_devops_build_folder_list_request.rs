use crate::azure_devops_build_page::AzureDevOpsBuildPage;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildFolder;
use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildFolderPath;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
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
use eyre::ensure;
use facet::Facet;
use reqwest::Method;
use reqwest::Url;
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;

/// A cached request for build definition folders.
///
/// Use `with_invalidation(true)` to refresh the matching inventory. Invalidation
/// clears every filter variant and page for the selected project, including
/// modern and legacy cloud URL aliases. Project ID and name selectors remain
/// distinct cache scopes; no project identity resolution is performed here.
///
/// See Microsoft's [Folders List API](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/folders/list?view=azure-devops-rest-7.1).
#[must_use = "This is a future request, you must .execute it"]
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsBuildFolderListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    /// Project ID or name.
    pub project: AzureDevOpsProjectArgument<'a>,
    pub auth_selection: AzureDevOpsAuthSelection,
    /// Optional starting folder path. Omit this for a complete project inventory.
    pub path: Option<AzureDevOpsBuildFolderPath>,
}

impl<'a> Arbitrary<'a> for AzureDevOpsBuildFolderListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
            auth_selection: AzureDevOpsAuthSelection::arbitrary(u)?,
            path: Option::<AzureDevOpsBuildFolderPath>::arbitrary(u)?,
        })
    }
}

impl HasCacheKey for AzureDevOpsBuildFolderListRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        let mut path = PathBuf::from("az/devops");
        if self.org_url.collection_name().is_some() {
            // Server scopes preserve their instance authority and path prefix.
            path.push("server");
            path.push(self.org_url.instance_scheme());
            for segment in self.org_url.instance_authority_and_path().split('/') {
                path.push(sanitize_windows_path_component(segment));
            }
            path.push("org");
            path.push(sanitize_windows_path_component(self.org_url.name()));
        } else {
            // Modern and legacy URLs invalidate the same readable cloud scope.
            path.push("org");
            path.push(sanitize_windows_path_component(
                &self.org_url.name().to_ascii_lowercase(),
            ));
        }
        let project = match &self.project {
            AzureDevOpsProjectArgument::Id(id) => id.to_string(),
            AzureDevOpsProjectArgument::Name(name) => {
                let name = name.to_string().to_ascii_lowercase();
                let encoded = sanitize_windows_path_component(&name);
                if name
                    .parse::<cloud_terrastodon_azure_devops_types::AzureDevOpsProjectId>()
                    .is_ok()
                {
                    // '=' is forbidden in project names, making this marker
                    // distinct from both ID selectors and ordinary names.
                    format!("name={encoded}")
                } else {
                    encoded
                }
            }
        };
        CacheKey::from_relative_path(
            path.join("project")
                .join(project)
                .join("build")
                .join("definition/folder/list"),
        )
    }
}

impl<'a> ExecutableRequest for AzureDevOpsBuildFolderListRequest<'a> {
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
    type Output = Vec<Arc<AzureDevOpsBuildFolder>>;

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let cache_key = self.cache_key()?;
            let mut base_url = Url::parse(&self.org_url.to_string())?;
            base_url
                .path_segments_mut()
                .map_err(|()| {
                    eyre::eyre!(
                        "Azure DevOps organization URL does not support hierarchical path segments"
                    )
                })?
                .pop_if_empty()
                .push(&self.project.to_string())
                .extend(["_apis", "build", "folders"]);
            if let Some(path) = &self.path {
                base_url
                    .path_segments_mut()
                    .map_err(|()| {
                        eyre::eyre!("Build folders URL does not support hierarchical path segments")
                    })?
                    .push(path.as_str());
            }
            base_url
                .query_pairs_mut()
                .append_pair("api-version", "7.1-preview.2");
            // The encoded request URL distinguishes project, path, and service
            // URL variants beneath the shared invalidation root.
            let cache_key =
                cache_key.join(blake3::hash(base_url.as_str().as_bytes()).to_hex().as_str())?;

            // Folders List does not document pagination. Honor any continuation
            // header nevertheless so an inventory cannot silently be incomplete.
            let mut folders = Vec::new();
            let mut continuation: Option<MicrosoftContinuationToken> = None;
            let mut seen = BTreeSet::new();
            let mut page_index = 0;
            loop {
                executor.cancellation().bail_if_cancelled()?;
                let mut url = base_url.clone();
                if let Some(token) = &continuation {
                    url.query_pairs_mut()
                        .append_pair("continuationToken", token.as_str());
                }
                let request = RestRequest::from_method_and_url(Method::GET, url)?
                    .cache(cache_key.join(page_index.to_string())?)
                    .authentication(self.auth_selection.into());
                let (page, next) =
                    request.receive_with_ms_continuation_token_cached::<AzureDevOpsBuildPage<Arc<AzureDevOpsBuildFolder>>>(&executor, CacheOperationOwner::of::<AzureDevOpsBuildFolderListRequest<'static>>()).await?;
                folders.extend(page.value.iter().cloned());
                let Some(token) = next else {
                    break;
                };
                ensure!(
                    seen.insert(token.clone()),
                    "Build API returned a repeated continuation token"
                );
                continuation = Some(token);
                page_index += 1;
            }
            Ok(folders)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildFolderListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildFolderListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureDevOpsBuildFolderListRequest<'static> => Vec<Arc<AzureDevOpsBuildFolder>>,
    effects = [Read]
);

impl InvalidatesOwnCacheKey for AzureDevOpsBuildFolderListRequest<'_> {}
#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectName;

    #[test]
    fn cloud_url_aliases_share_folder_invalidation_root() {
        let modern = "https://dev.azure.com/synthetic".parse().unwrap();
        let legacy = "https://synthetic.visualstudio.com".parse().unwrap();
        let project: AzureDevOpsProjectArgument<'static> = "Synthetic project".parse().unwrap();
        let auth_selection = crate::test_executor::authentication_selection();
        assert_eq!(
            AzureDevOpsBuildFolderListRequest {
                org_url: Cow::Borrowed(&modern),
                project: (project.clone()),
                auth_selection,
                path: (None)
            }
            .cache_key()
            .unwrap(),
            AzureDevOpsBuildFolderListRequest {
                org_url: Cow::Borrowed(&legacy),
                project: (project),
                auth_selection,
                path: (None)
            }
            .cache_key()
            .unwrap()
        );
        let key = AzureDevOpsBuildFolderListRequest {
            org_url: Cow::Borrowed(&modern),
            project: ("Synthetic project"
                .parse::<AzureDevOpsProjectArgument>()
                .unwrap()),
            auth_selection,
            path: (None),
        }
        .cache_key()
        .unwrap();
        assert_eq!(
            key.scope.components().iter().collect::<PathBuf>(),
            PathBuf::from(
                "az/devops/org/synthetic/project/synthetic%20project/build/definition/folder/list"
            )
        );
    }

    #[test]
    fn server_cache_scopes_include_the_instance_and_collection() {
        let urls = [
            "https://one.example.invalid/tfs/Collection",
            "https://two.example.invalid/tfs/Collection",
            "https://one.example.invalid:8443/tfs/Collection",
            "https://one.example.invalid/other/Collection",
            "https://one.example.invalid/tfs/Other",
        ];
        let project: AzureDevOpsProjectArgument<'static> = "Synthetic project".parse().unwrap();
        let auth_selection = crate::test_executor::authentication_selection();
        let keys = urls.map(|url| {
            let organization = url.parse().unwrap();
            AzureDevOpsBuildFolderListRequest {
                org_url: Cow::Borrowed(&organization),
                project: (project.clone()),
                auth_selection,
                path: (None),
            }
            .cache_key()
            .unwrap()
        });
        for (index, key) in keys.iter().enumerate() {
            assert!(!keys[..index].contains(key));
        }
        assert_eq!(
            keys[0].scope.components().iter().collect::<PathBuf>(),
            PathBuf::from(
                "az/devops/server/https/one.example.invalid/tfs/org/Collection/project/synthetic%20project/build/definition/folder/list"
            )
        );
        assert_eq!(
            keys[2].scope.components().iter().collect::<PathBuf>(),
            PathBuf::from(
                "az/devops/server/https/one.example.invalid%3A8443/tfs/org/Collection/project/synthetic%20project/build/definition/folder/list"
            )
        );
    }

    #[test]
    fn folder_cache_invalidation_is_scoped_to_the_project_selector() {
        let organization = "https://dev.azure.com/synthetic".parse().unwrap();
        let auth_selection = crate::test_executor::authentication_selection();
        let project: AzureDevOpsProjectArgument<'static> = "Synthetic project".parse().unwrap();
        let same_project: AzureDevOpsProjectArgument<'static> =
            "SYNTHETIC PROJECT".parse().unwrap();
        let other_project: AzureDevOpsProjectArgument<'static> = "Other project".parse().unwrap();
        let key = AzureDevOpsBuildFolderListRequest {
            org_url: Cow::Borrowed(&organization),
            project: (project.clone()),
            auth_selection,
            path: (None),
        }
        .cache_key()
        .unwrap();
        assert_eq!(
            key,
            AzureDevOpsBuildFolderListRequest {
                org_url: Cow::Borrowed(&organization),
                project: (same_project),
                auth_selection,
                path: (Some(AzureDevOpsBuildFolderPath::try_new(r"\Synthetic\CI").unwrap()))
            }
            .cache_key()
            .unwrap()
        );
        assert_ne!(
            key,
            AzureDevOpsBuildFolderListRequest {
                org_url: Cow::Borrowed(&organization),
                project: (other_project),
                auth_selection,
                path: (None)
            }
            .cache_key()
            .unwrap()
        );

        let id_text = "11111111-1111-1111-1111-111111111111";
        let project_id: AzureDevOpsProjectArgument<'static> = id_text.parse().unwrap();
        let project_name = AzureDevOpsProjectName::try_new(id_text).unwrap();
        assert_ne!(
            AzureDevOpsBuildFolderListRequest {
                org_url: Cow::Borrowed(&organization),
                project: (project_id),
                auth_selection,
                path: (None)
            }
            .cache_key()
            .unwrap(),
            AzureDevOpsBuildFolderListRequest {
                org_url: Cow::Borrowed(&organization),
                project: (project_name).into(),
                auth_selection,
                path: (None)
            }
            .cache_key()
            .unwrap()
        );
    }

    #[test]
    fn readable_project_scopes_preserve_spaces_underscores_and_percent_escapes() {
        let organization = "https://dev.azure.com/synthetic".parse().unwrap();
        let auth_selection = crate::test_executor::authentication_selection();
        let names = [
            "Synthetic project",
            "Synthetic_project",
            "Synthetic%20project",
        ];
        let keys = names.map(|name| {
            let project = AzureDevOpsProjectName::try_new(name).unwrap();
            AzureDevOpsBuildFolderListRequest {
                org_url: Cow::Borrowed(&organization),
                project: (project).into(),
                auth_selection,
                path: (None),
            }
            .cache_key()
            .unwrap()
            .scope
        });
        for (index, key) in keys.iter().enumerate() {
            assert!(!keys[..index].contains(key));
        }
        assert!(
            keys[0]
                .components()
                .iter()
                .any(|component| component == "synthetic%20project")
        );
        assert!(
            keys[1]
                .components()
                .iter()
                .any(|component| component == "synthetic_project")
        );
        assert!(
            keys[2]
                .components()
                .iter()
                .any(|component| component == "synthetic%2520project")
        );
    }
}

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureDevOpsBuildFolderListRequest<'static>,
    name = "azure-devops-build-folder-list",
    cached
);
