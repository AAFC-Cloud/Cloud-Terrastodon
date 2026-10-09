use crate::azure_devops_build_page::AzureDevOpsBuildPage;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildDefinitionReference;
use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildFolderPath;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_command::CacheInvalidatable;
use cloud_terrastodon_command::CacheInvalidatableIntoFuture;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::HasCacheKey;
use cloud_terrastodon_credentials::AzureDevOpsAuthContext;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use cloud_terrastodon_rest::MicrosoftContinuationToken;
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

/// A cached request for every page of matching build definition references.
///
/// Use `with_invalidation(true)` to refresh the matching inventory. Invalidation
/// clears every filter variant and page for the selected project, including
/// modern and legacy cloud URL aliases. Project ID and name selectors remain
/// distinct cache scopes; no project identity resolution is performed here.
///
/// See Microsoft's [Definitions List API](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/list?view=azure-devops-rest-7.1).
#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsBuildDefinitionListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    /// Project ID or name.
    pub project: AzureDevOpsProjectArgument<'a>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
    /// Definition name or wildcard name filter.
    pub name: Option<String>,
    pub path: Option<AzureDevOpsBuildFolderPath>,
}

/// List build definition references using the Azure DevOps 7.1 API.
///
/// See Microsoft's [Definitions List API](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/list?view=azure-devops-rest-7.1).
pub fn fetch_azure_devops_build_definitions<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    project: impl Into<AzureDevOpsProjectArgument<'a>>,
    auth_context: &'a AzureDevOpsAuthContext,
    name: Option<String>,
    path: Option<AzureDevOpsBuildFolderPath>,
) -> AzureDevOpsBuildDefinitionListRequest<'a> {
    AzureDevOpsBuildDefinitionListRequest {
        org_url: Cow::Borrowed(org_url),
        project: project.into(),
        auth_context: Cow::Borrowed(auth_context),
        name,
        path,
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsBuildDefinitionListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
            name: u.arbitrary()?,
            path: u.arbitrary()?,
        })
    }
}

impl HasCacheKey for AzureDevOpsBuildDefinitionListRequest<'_> {
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
                .join("definition/list"),
        )
    }
}

impl<'a> CacheInvalidatableIntoFuture for AzureDevOpsBuildDefinitionListRequest<'a> {
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

impl<'a> IntoFuture for AzureDevOpsBuildDefinitionListRequest<'a> {
    type Output = Result<Vec<AzureDevOpsBuildDefinitionReference>>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let cache_key = self.cache_key();
            let mut url = Url::parse(&self.org_url.to_string())?;
            url.path_segments_mut()
                .map_err(|()| {
                    eyre::eyre!(
                        "Azure DevOps organization URL does not support hierarchical path segments"
                    )
                })?
                .pop_if_empty()
                .push(&self.project.to_string())
                .extend(["_apis", "build", "definitions"]);
            url.query_pairs_mut().append_pair("api-version", "7.1");
            if let Some(name) = &self.name {
                url.query_pairs_mut().append_pair("name", name);
            }
            if let Some(path) = &self.path {
                url.query_pairs_mut().append_pair("path", path.as_str());
            }
            // Keep every encoded filter variant beneath the shared
            // selected project's invalidation root.
            let cache_key = cache_key.join(blake3::hash(url.as_str().as_bytes()).to_hex().as_str());

            let mut definitions = Vec::new();
            let mut continuation: Option<MicrosoftContinuationToken> = None;
            let mut seen = BTreeSet::new();
            let mut page_index = 0;
            loop {
                let mut page_url = url.clone();
                if let Some(token) = &continuation {
                    page_url
                        .query_pairs_mut()
                        .append_pair("continuationToken", token.as_str());
                }
                let request = RestRequest::from_method_and_url(Method::GET, page_url)?
                    .cache(cache_key.join(page_index.to_string()))
                    .azure_devops_auth_context(self.auth_context.as_ref())?;
                let (page, next): (AzureDevOpsBuildPage<AzureDevOpsBuildDefinitionReference>, _) =
                    request.receive_with_ms_continuation_token().await?;
                definitions.extend(page.value);
                // An empty page can still carry a valid continuation token.
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
            Ok(definitions)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildDefinitionListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildDefinitionListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(
    AzureDevOpsBuildDefinitionListRequest<'static> => Vec<AzureDevOpsBuildDefinitionReference>,
    effects = [Read]
);
