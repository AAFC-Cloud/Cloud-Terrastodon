use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops_types::AzureDevOpsDescriptor;
use cloud_terrastodon_azure_devops_types::AzureDevOpsGroupMembersV2Response;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_rest::RestRequest;
use facet::Facet;
use reqwest::Method;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsGroupMembersV2Request<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub group_id: Cow<'a, AzureDevOpsDescriptor>,
    pub auth_selection: AzureDevOpsAuthSelection,
}

impl<'a> Arbitrary<'a> for AzureDevOpsGroupMembersV2Request<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            group_id: Cow::Owned(AzureDevOpsDescriptor::arbitrary(u)?),
            auth_selection: AzureDevOpsAuthSelection::arbitrary(u)?,
        })
    }
}
impl<'a> HasCacheKey for AzureDevOpsGroupMembersV2Request<'a> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "devops",
            self.org_url.name(),
            "graph",
            "memberships",
            self.group_id.to_string().as_ref(),
            "down",
        ]))
    }
}

impl<'a> InvalidatesOwnCacheKey for AzureDevOpsGroupMembersV2Request<'a> {}

impl<'a> ExecutableRequest for AzureDevOpsGroupMembersV2Request<'a> {
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
    type Output = Arc<AzureDevOpsGroupMembersV2Response>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let organization = self.org_url.organization_name()?;
            let subject_descriptor = self.group_id.as_ref();
            let url = format!(
                "https://vssps.dev.azure.com/{organization}/_apis/graph/Memberships/{subject_descriptor}?api-version=7.1-preview.1&direction=down",
                organization = organization,
                subject_descriptor = subject_descriptor
            );
            let request = RestRequest::new(Method::GET, url)?.cache(self.cache_key()?);
            let response = request
                .authentication(self.auth_selection.into())
                .receive_cached::<AzureDevOpsGroupMembersV2Response>(
                    &executor,
                    CacheOperationOwner::of::<AzureDevOpsGroupMembersV2Request<'static>>(),
                )
                .await?;
            Ok(response)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsGroupMembersV2Request<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsGroupMembersV2Request<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureDevOpsGroupMembersV2Request<'static> => Arc<AzureDevOpsGroupMembersV2Response>,
    effects = [Read]
);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureDevOpsGroupMembersV2Request<'static>,
    name = "azure-devops-group-members-v2",
    cached
);
