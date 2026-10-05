use crate::azure_devops_build_page::AzureDevOpsBuildPage;
use crate::azure_devops_rest::authenticate_azure_devops_request;
use crate::azure_devops_rest::page_cache_key;
use crate::azure_devops_rest::receive_azure_devops_page;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildFolder;
use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildFolderPath;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_command::CacheInvalidatable;
use cloud_terrastodon_command::CacheInvalidatableIntoFuture;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::HasCacheKey;
use cloud_terrastodon_credentials::AzureDevOpsAuthContext;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use eyre::ensure;
use facet::Facet;
use reqwest::Method;
use reqwest::Url;
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::future::Future;
use std::future::IntoFuture;
use std::path::PathBuf;
use std::pin::Pin;

/// A cached request for build definition folders.
///
/// Use `with_invalidation(true)` to refresh the matching inventory. Invalidation
/// clears every filter variant and page for the selected project, including
/// modern and legacy cloud URL aliases. Project ID and name selectors remain
/// distinct cache scopes; no project identity resolution is performed here.
///
/// See Microsoft's [Folders List API](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/folders/list?view=azure-devops-rest-7.1).
#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsBuildFolderListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project: AzureDevOpsProjectArgument<'a>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
    /// Optional starting folder path. Omit this for a complete project inventory.
    pub path: Option<AzureDevOpsBuildFolderPath>,
}

/// List build definition folders using the Azure DevOps 7.1-preview.2 API.
///
/// See Microsoft's [Folders List API](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/folders/list?view=azure-devops-rest-7.1).
pub fn fetch_azure_devops_build_folders<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    project: impl Into<AzureDevOpsProjectArgument<'a>>,
    auth_context: &'a AzureDevOpsAuthContext,
    path: Option<AzureDevOpsBuildFolderPath>,
) -> AzureDevOpsBuildFolderListRequest<'a> {
    AzureDevOpsBuildFolderListRequest {
        org_url: Cow::Borrowed(org_url),
        project: project.into(),
        auth_context: Cow::Borrowed(auth_context),
        path,
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsBuildFolderListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
            path: Option::<AzureDevOpsBuildFolderPath>::arbitrary(u)?,
        })
    }
}

impl HasCacheKey for AzureDevOpsBuildFolderListRequest<'_> {
    fn cache_key(&self) -> CacheKey {
        let mut path = PathBuf::from("az/devops");
        if self.org_url.collection_name().is_some() {
            // Server scopes preserve their instance authority and path prefix.
            let (scheme, location) = self
                .org_url
                .base_url()
                .split_once("://")
                .expect("validated Server URLs have a scheme and authority");
            let mut segments = location.split('/');
            path.push("server");
            path.push(scheme);
            path.push(sanitize_windows_path_component(
                segments
                    .next()
                    .expect("validated Server URLs have an authority"),
            ));
            for segment in segments {
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
        CacheKey::new(
            path.join("project")
                .join(project)
                .join("build")
                .join("definition/folder/list"),
        )
    }
}

impl<'a> CacheInvalidatableIntoFuture for AzureDevOpsBuildFolderListRequest<'a> {
    type WithInvalidation = <Self as IntoFuture>::IntoFuture;

    fn with_invalidation(self, invalidate_cache: bool) -> Self::WithInvalidation {
        Box::pin(async move {
            if invalidate_cache {
                self.invalidate().await?;
            }
            self.into_future().await
        })
    }
}

impl<'a> IntoFuture for AzureDevOpsBuildFolderListRequest<'a> {
    type Output = Result<Vec<AzureDevOpsBuildFolder>>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let mut cache_key = self.cache_key();
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
            cache_key.path = cache_key
                .path
                .join(blake3::hash(base_url.as_str().as_bytes()).to_hex().as_str());

            // Folders List does not document pagination. Honor any continuation
            // header nevertheless so an inventory cannot silently be incomplete.
            let mut folders = Vec::new();
            let mut continuation: Option<String> = None;
            let mut seen = BTreeSet::new();
            let mut page_index = 0;
            loop {
                let mut url = base_url.clone();
                if let Some(token) = &continuation {
                    url.query_pairs_mut()
                        .append_pair("continuationToken", token);
                }
                let request = authenticate_azure_devops_request(
                    RestRequest::new(Method::GET, url.as_str())?
                        .cache(page_cache_key(&cache_key, page_index)),
                    &self.auth_context,
                )?;
                let (page, next): (AzureDevOpsBuildPage<AzureDevOpsBuildFolder>, _) =
                    receive_azure_devops_page(request).await?;
                folders.extend(page.value);
                let Some(token) = next else {
                    break;
                };
                ensure!(
                    !token.trim().is_empty(),
                    "Build API returned an empty continuation token"
                );
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
cloud_terrastodon_registry::register_into_future!(
    AzureDevOpsBuildFolderListRequest<'static> => Vec<AzureDevOpsBuildFolder>,
    effects = [Read]
);

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectName;
    use cloud_terrastodon_command::NoSpaces;

    #[test]
    fn cloud_url_aliases_share_folder_invalidation_root() {
        let modern = "https://dev.azure.com/synthetic".parse().unwrap();
        let legacy = "https://synthetic.visualstudio.com".parse().unwrap();
        let project: AzureDevOpsProjectArgument<'static> = "Synthetic project".parse().unwrap();
        let auth_context = AzureDevOpsAuthContext::None;
        assert_eq!(
            fetch_azure_devops_build_folders(&modern, project.clone(), &auth_context, None)
                .cache_key(),
            fetch_azure_devops_build_folders(&legacy, project, &auth_context, None).cache_key()
        );
        let key = fetch_azure_devops_build_folders(
            &modern,
            "Synthetic project"
                .parse::<AzureDevOpsProjectArgument>()
                .unwrap(),
            &auth_context,
            None,
        )
        .cache_key();
        assert_eq!(
            key.path,
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
        let auth_context = AzureDevOpsAuthContext::None;
        let keys = urls.map(|url| {
            let organization = url.parse().unwrap();
            fetch_azure_devops_build_folders(&organization, project.clone(), &auth_context, None)
                .cache_key()
        });
        for (index, key) in keys.iter().enumerate() {
            assert!(!keys[..index].contains(key));
        }
        assert_eq!(
            keys[0].path,
            PathBuf::from(
                "az/devops/server/https/one.example.invalid/tfs/org/Collection/project/synthetic%20project/build/definition/folder/list"
            )
        );
        assert_eq!(
            keys[2].path,
            PathBuf::from(
                "az/devops/server/https/one.example.invalid%3A8443/tfs/org/Collection/project/synthetic%20project/build/definition/folder/list"
            )
        );
    }

    #[test]
    fn folder_cache_invalidation_is_scoped_to_the_project_selector() {
        let organization = "synthetic".parse().unwrap();
        let auth_context = AzureDevOpsAuthContext::None;
        let project: AzureDevOpsProjectArgument<'static> = "Synthetic project".parse().unwrap();
        let same_project: AzureDevOpsProjectArgument<'static> =
            "SYNTHETIC PROJECT".parse().unwrap();
        let other_project: AzureDevOpsProjectArgument<'static> = "Other project".parse().unwrap();
        let key =
            fetch_azure_devops_build_folders(&organization, project.clone(), &auth_context, None)
                .cache_key();
        assert_eq!(
            key,
            fetch_azure_devops_build_folders(
                &organization,
                same_project,
                &auth_context,
                Some(AzureDevOpsBuildFolderPath::try_new(r"\Synthetic\CI").unwrap()),
            )
            .cache_key()
        );
        assert_ne!(
            key,
            fetch_azure_devops_build_folders(&organization, other_project, &auth_context, None)
                .cache_key()
        );

        let id_text = "11111111-1111-1111-1111-111111111111";
        let project_id: AzureDevOpsProjectArgument<'static> = id_text.parse().unwrap();
        let project_name = AzureDevOpsProjectName::try_new(id_text).unwrap();
        assert_ne!(
            fetch_azure_devops_build_folders(&organization, project_id, &auth_context, None)
                .cache_key(),
            fetch_azure_devops_build_folders(&organization, project_name, &auth_context, None)
                .cache_key()
        );
    }

    #[test]
    fn readable_project_scopes_survive_no_spaces_without_identity_collisions() {
        let organization = "synthetic".parse().unwrap();
        let auth_context = AzureDevOpsAuthContext::None;
        let names = [
            "Synthetic project",
            "Synthetic_project",
            "Synthetic%20project",
        ];
        let keys = names.map(|name| {
            let project = AzureDevOpsProjectName::try_new(name).unwrap();
            fetch_azure_devops_build_folders(&organization, project, &auth_context, None)
                .cache_key()
                .path
                .no_spaces()
        });
        for (index, key) in keys.iter().enumerate() {
            assert!(!keys[..index].contains(key));
        }
        assert!(keys[0].to_string_lossy().contains("synthetic%20project"));
        assert!(keys[1].to_string_lossy().contains("synthetic_project"));
        assert!(keys[2].to_string_lossy().contains("synthetic%2520project"));
    }
}
