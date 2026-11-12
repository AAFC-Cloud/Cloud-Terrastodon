use crate::azure_devops_build_page::AzureDevOpsBuildPage;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops_types::AzureDevOpsBuild;
use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildDefinitionId;
use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildListLimit;
use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildResult;
use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildStatus;
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

/// A cached request for build runs, newest queue time first.
///
/// Use `with_invalidation(true)` to refresh the matching inventory. Invalidation
/// clears every filter variant and page for the selected project, including
/// modern and legacy cloud URL aliases. Project ID and name selectors remain
/// distinct cache scopes; no project identity resolution is performed here.
///
/// See Microsoft's [Builds List API](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/builds/list?view=azure-devops-rest-7.1).
#[must_use = "This is a future request, you must .execute it"]
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsBuildListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    /// Project ID or name.
    pub project: AzureDevOpsProjectArgument<'a>,
    pub auth_selection: AzureDevOpsAuthSelection,
    pub definitions: Vec<AzureDevOpsBuildDefinitionId>,
    pub status: Option<AzureDevOpsBuildStatus>,
    pub result: Option<AzureDevOpsBuildResult>,
    pub branch: Option<String>,
    /// A positive total limit, or `None` to retrieve every returned build.
    pub limit: Option<AzureDevOpsBuildListLimit>,
}

impl<'a> Arbitrary<'a> for AzureDevOpsBuildListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
            auth_selection: AzureDevOpsAuthSelection::arbitrary(u)?,
            definitions: u.arbitrary()?,
            status: u.arbitrary()?,
            result: u.arbitrary()?,
            branch: u.arbitrary()?,
            limit: u.arbitrary()?,
        })
    }
}

impl HasCacheKey for AzureDevOpsBuildListRequest<'_> {
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
                .join("list"),
        )
    }
}

impl<'a> ExecutableRequest for AzureDevOpsBuildListRequest<'a> {
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
    type Output = Vec<Arc<AzureDevOpsBuild>>;

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let cache_key = self.cache_key()?;
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
            )?;
            let mut builds = Vec::new();
            let mut continuation: Option<MicrosoftContinuationToken> = None;
            let mut seen = BTreeSet::new();
            let mut page_index = 0;
            loop {
                executor.cancellation().bail_if_cancelled()?;
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
                    .cache(cache_key.join(page_index.to_string())?)
                    .authentication(self.auth_selection.into());
                let (page, next) =
                    request.receive_with_ms_continuation_token_cached::<AzureDevOpsBuildPage<Arc<AzureDevOpsBuild>>>(&executor, CacheOperationOwner::of::<AzureDevOpsBuildListRequest<'static>>()).await?;
                builds.extend(
                    page.value
                        .iter()
                        .take(remaining.unwrap_or(usize::MAX))
                        .cloned(),
                );
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
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureDevOpsBuildListRequest<'static> => Vec<Arc<AzureDevOpsBuild>>,
    effects = [Read]
);

impl InvalidatesOwnCacheKey for AzureDevOpsBuildListRequest<'_> {}

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureDevOpsBuildListRequest<'static>,
    name = "azure-devops-build-list",
    cached
);
