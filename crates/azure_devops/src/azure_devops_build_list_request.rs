use crate::azure_devops_build_page::AzureDevOpsBuildPage;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_devops_types::AzureDevOpsBuild;
use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildDefinitionId;
use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildListLimit;
use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildResult;
use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildStatus;
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

/// A cached request for build runs, newest queue time first.
///
/// Use `with_invalidation(true)` to refresh the matching inventory. Invalidation
/// clears every filter variant and page for the selected project, including
/// modern and legacy cloud URL aliases. Project ID and name selectors remain
/// distinct cache scopes; no project identity resolution is performed here.
///
/// See Microsoft's [Builds List API](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/builds/list?view=azure-devops-rest-7.1).
#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsBuildListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project: AzureDevOpsProjectArgument<'a>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
    pub definitions: Vec<AzureDevOpsBuildDefinitionId>,
    pub status: Option<AzureDevOpsBuildStatus>,
    pub result: Option<AzureDevOpsBuildResult>,
    pub branch: Option<String>,
    /// A positive total limit, or `None` to retrieve every returned build.
    pub limit: Option<AzureDevOpsBuildListLimit>,
}

/// List build runs using the Azure DevOps 7.1 API, with a total limit across pages.
///
/// See Microsoft's [Builds List API](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/builds/list?view=azure-devops-rest-7.1).
#[expect(
    clippy::too_many_arguments,
    reason = "Expose build-list filters directly instead of a convenience parameter wrapper"
)]
pub fn fetch_azure_devops_builds<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    project: impl Into<AzureDevOpsProjectArgument<'a>>,
    auth_context: &'a AzureDevOpsAuthContext,
    definitions: Vec<AzureDevOpsBuildDefinitionId>,
    status: Option<AzureDevOpsBuildStatus>,
    result: Option<AzureDevOpsBuildResult>,
    branch: Option<String>,
    limit: Option<AzureDevOpsBuildListLimit>,
) -> AzureDevOpsBuildListRequest<'a> {
    AzureDevOpsBuildListRequest {
        org_url: Cow::Borrowed(org_url),
        project: project.into(),
        auth_context: Cow::Borrowed(auth_context),
        definitions,
        status,
        result,
        branch,
        limit,
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsBuildListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
            definitions: u.arbitrary()?,
            status: u.arbitrary()?,
            result: u.arbitrary()?,
            branch: u.arbitrary()?,
            limit: u.arbitrary()?,
        })
    }
}

impl HasCacheKey for AzureDevOpsBuildListRequest<'_> {
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
                .join("list"),
        )
    }
}

impl<'a> CacheInvalidatableIntoFuture for AzureDevOpsBuildListRequest<'a> {
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

impl<'a> IntoFuture for AzureDevOpsBuildListRequest<'a> {
    type Output = Result<Vec<AzureDevOpsBuild>>;
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
                .extend(["_apis", "build", "builds"]);
            url.query_pairs_mut()
                .append_pair("api-version", "7.1")
                .append_pair("queryOrder", "queueTimeDescending");
            if !self.definitions.is_empty() {
                url.query_pairs_mut().append_pair(
                    "definitions",
                    &self
                        .definitions
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                );
            }
            if let Some(status) = self.status {
                url.query_pairs_mut()
                    .append_pair("statusFilter", status.as_str());
            }
            if let Some(result) = self.result {
                url.query_pairs_mut()
                    .append_pair("resultFilter", result.as_str());
            }
            if let Some(branch) = &self.branch {
                url.query_pairs_mut().append_pair("branchName", branch);
            }

            let limit = self
                .limit
                .map(|limit| usize::try_from(limit.get()))
                .transpose()?;
            // Hash the complete initial request, including its total limit.
            // Later pages use the remaining count without changing this key.
            let mut cache_url = url.clone();
            if let Some(limit) = limit {
                cache_url
                    .query_pairs_mut()
                    .append_pair("$top", &limit.to_string());
            }
            let cache_key = cache_key.join(
                blake3::hash(cache_url.as_str().as_bytes())
                    .to_hex()
                    .as_str(),
            );
            let mut builds = Vec::new();
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
                // `$top` limits each server response. Keep it equal to the
                // remaining total so callers receive at most the requested count.
                let remaining = limit.map(|limit| limit - builds.len());
                if let Some(remaining) = remaining {
                    page_url
                        .query_pairs_mut()
                        .append_pair("$top", &remaining.to_string());
                }
                let request = RestRequest::from_method_and_url(Method::GET, page_url)?
                    .cache(cache_key.join(page_index.to_string()))
                    .azure_devops_auth_context(self.auth_context.as_ref())?;
                let (page, next): (AzureDevOpsBuildPage<AzureDevOpsBuild>, _) =
                    request.receive_with_ms_continuation_token().await?;
                builds.extend(page.value.into_iter().take(remaining.unwrap_or(usize::MAX)));
                if limit.is_some_and(|limit| builds.len() == limit) {
                    break;
                }
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
            Ok(builds)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(
    AzureDevOpsBuildListRequest<'static> => Vec<AzureDevOpsBuild>,
    effects = [Read]
);
