use crate::azure_devops_build_page::AzureDevOpsBuildPage;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildDefinitionReference;
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

/// A cached request for every page of matching build definition references.
///
/// Use `with_invalidation(true)` to refresh the matching inventory. Invalidation
/// clears every filter variant and page for the selected project, including
/// modern and legacy cloud URL aliases. Project ID and name selectors remain
/// distinct cache scopes; no project identity resolution is performed here.
///
/// See Microsoft's [Definitions List API](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/list?view=azure-devops-rest-7.1).
#[must_use = "This is a future request, you must .execute it"]
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsBuildDefinitionListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    /// Project ID or name.
    pub project: AzureDevOpsProjectArgument<'a>,
    pub auth_selection: AzureDevOpsAuthSelection,
    /// Definition name or wildcard name filter.
    pub name: Option<String>,
    pub path: Option<AzureDevOpsBuildFolderPath>,
}

impl<'a> Arbitrary<'a> for AzureDevOpsBuildDefinitionListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
            auth_selection: AzureDevOpsAuthSelection::arbitrary(u)?,
            name: u.arbitrary()?,
            path: u.arbitrary()?,
        })
    }
}

impl HasCacheKey for AzureDevOpsBuildDefinitionListRequest<'_> {
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
                .join("definition/list"),
        )
    }
}

impl<'a> ExecutableRequest for AzureDevOpsBuildDefinitionListRequest<'a> {
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
    type Output = Vec<Arc<AzureDevOpsBuildDefinitionReference>>;

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
            let cache_key =
                cache_key.join(blake3::hash(url.as_str().as_bytes()).to_hex().as_str())?;

            let mut definitions = Vec::new();
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
                let request = RestRequest::from_method_and_url(Method::GET, page_url)?
                    .cache(cache_key.join(page_index.to_string())?)
                    .authentication(self.auth_selection.into());
                let (page, next) =
                    request.receive_with_ms_continuation_token_cached::<AzureDevOpsBuildPage<Arc<AzureDevOpsBuildDefinitionReference>>>(&executor, CacheOperationOwner::of::<AzureDevOpsBuildDefinitionListRequest<'static>>()).await?;
                definitions.extend(page.value.iter().cloned());
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
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureDevOpsBuildDefinitionListRequest<'static> => Vec<Arc<AzureDevOpsBuildDefinitionReference>>,
    effects = [Read]
);

impl InvalidatesOwnCacheKey for AzureDevOpsBuildDefinitionListRequest<'_> {}

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureDevOpsBuildDefinitionListRequest<'static>,
    name = "azure-devops-build-definition-list",
    cached
);
