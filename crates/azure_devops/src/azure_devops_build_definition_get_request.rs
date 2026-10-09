use arbitrary::Arbitrary;
use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildDefinition;
use cloud_terrastodon_azure_devops_types::AzureDevOpsBuildDefinitionId;
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
use std::future::Future;
use std::future::IntoFuture;
use std::path::PathBuf;
use std::pin::Pin;

/// A cached request for one complete build definition, optionally at a revision.
///
/// Unlike the list endpoint's references, this returns the definition's process
/// and repository configuration. Omitting `revision` selects its latest version.
/// Use `with_invalidation(true)` to refresh the selected definition: invalidation
/// clears its latest and revision entries in this project, including modern and
/// legacy cloud URL aliases. Project ID and name selectors remain distinct scopes.
///
/// See Microsoft's [Definitions Get API](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/get?view=azure-devops-rest-7.1).
#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsBuildDefinitionGetRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    /// Project ID or name.
    pub project: AzureDevOpsProjectArgument<'a>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
    pub definition_id: AzureDevOpsBuildDefinitionId,
    /// The revision to retrieve; `None` retrieves the latest definition.
    pub revision: Option<i32>,
}

/// Retrieve a complete build definition using the Azure DevOps 7.1 API.
///
/// See Microsoft's [Definitions Get API](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/get?view=azure-devops-rest-7.1).
pub fn fetch_azure_devops_build_definition<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    project: impl Into<AzureDevOpsProjectArgument<'a>>,
    auth_context: &'a AzureDevOpsAuthContext,
    definition_id: AzureDevOpsBuildDefinitionId,
    revision: Option<i32>,
) -> AzureDevOpsBuildDefinitionGetRequest<'a> {
    AzureDevOpsBuildDefinitionGetRequest {
        org_url: Cow::Borrowed(org_url),
        project: project.into(),
        auth_context: Cow::Borrowed(auth_context),
        definition_id,
        revision,
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsBuildDefinitionGetRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
            definition_id: u.arbitrary()?,
            revision: u.arbitrary()?,
        })
    }
}

impl HasCacheKey for AzureDevOpsBuildDefinitionGetRequest<'_> {
    fn cache_key(&self) -> CacheKey {
        let mut path = PathBuf::from("az/devops");
        if self.org_url.collection_name().is_some() {
            // Preserve the Server instance scheme, authority and path prefix.
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
            // Modern and legacy URLs share the same readable cloud scope.
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
                    // '=' is forbidden in project names, distinguishing names
                    // that resemble UUIDs from actual project ID selectors.
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
                .join("definition/show")
                .join(self.definition_id.to_string()),
        )
    }
}

impl<'a> CacheInvalidatableIntoFuture for AzureDevOpsBuildDefinitionGetRequest<'a> {
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

impl<'a> IntoFuture for AzureDevOpsBuildDefinitionGetRequest<'a> {
    type Output = Result<AzureDevOpsBuildDefinition>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let mut url = Url::parse(&self.org_url.to_string())?;
            url.path_segments_mut()
                .map_err(|()| {
                    eyre::eyre!(
                        "Azure DevOps organization URL does not support hierarchical path segments"
                    )
                })?
                .pop_if_empty()
                .push(&self.project.to_string())
                .extend(["_apis", "build", "definitions"])
                .push(&self.definition_id.to_string());
            url.query_pairs_mut().append_pair("api-version", "7.1");
            // Keep revision variants beneath this definition's invalidation root.
            let cache_key = if let Some(revision) = self.revision {
                url.query_pairs_mut()
                    .append_pair("revision", &revision.to_string());
                self.cache_key().join("revision").join(revision.to_string())
            } else {
                self.cache_key().join("latest")
            };
            let request = RestRequest::from_method_and_url(Method::GET, url)?
                .cache(cache_key)
                .azure_devops_auth_context(self.auth_context.as_ref())?;
            let definition_id = self.definition_id;
            request
                .receive_with_validator(move |definition: AzureDevOpsBuildDefinition| {
                    ensure!(
                        definition.id == definition_id,
                        "Build definition response did not match the requested ID"
                    );
                    Ok(definition)
                })
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildDefinitionGetRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildDefinitionGetRequest<'static>);
cloud_terrastodon_registry::register_into_future!(
    AzureDevOpsBuildDefinitionGetRequest<'static> => AzureDevOpsBuildDefinition,
    effects = [Read]
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn definition_refresh_scope_covers_revisions_and_url_aliases_but_isolates_resources()
    -> Result<()> {
        let mut request = AzureDevOpsBuildDefinitionGetRequest {
            org_url: Cow::Owned("https://dev.azure.com/synthetic".parse()?),
            project: "Synthetic project".parse()?,
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
            definition_id: AzureDevOpsBuildDefinitionId::new(42)?,
            revision: None,
        };
        let key = request.cache_key();
        assert_eq!(
            key.path,
            PathBuf::from(
                "az/devops/org/synthetic/project/synthetic%20project/build/definition/show/42"
            )
        );

        request.revision = Some(3);
        assert_eq!(request.cache_key(), key);
        request.org_url = Cow::Owned("https://synthetic.visualstudio.com".parse()?);
        assert_eq!(request.cache_key(), key);

        request.definition_id = AzureDevOpsBuildDefinitionId::new(43)?;
        assert!(!request.cache_key().path.starts_with(&key.path));
        request.definition_id = AzureDevOpsBuildDefinitionId::new(42)?;
        request.project = "Other project".parse()?;
        assert!(!request.cache_key().path.starts_with(&key.path));
        request.project = "Synthetic project".parse()?;
        request.org_url = Cow::Owned("https://dev.azure.com/other-synthetic".parse()?);
        assert!(!request.cache_key().path.starts_with(&key.path));
        Ok(())
    }
}
