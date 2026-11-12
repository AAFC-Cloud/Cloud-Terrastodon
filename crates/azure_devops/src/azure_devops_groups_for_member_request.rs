use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops_types::AzureDevOpsCollectionResponse;
use cloud_terrastodon_azure_devops_types::AzureDevOpsDescriptor;
use cloud_terrastodon_azure_devops_types::AzureDevOpsGraphMembership;
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
pub struct AzureDevOpsGroupsForMemberRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub member_id: Cow<'a, AzureDevOpsDescriptor>,
    pub auth_selection: AzureDevOpsAuthSelection,
}

impl<'a> Arbitrary<'a> for AzureDevOpsGroupsForMemberRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            member_id: Cow::Owned(AzureDevOpsDescriptor::arbitrary(u)?),
            auth_selection: AzureDevOpsAuthSelection::arbitrary(u)?,
        })
    }
}

impl<'a> HasCacheKey for AzureDevOpsGroupsForMemberRequest<'a> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "devops",
            self.org_url.name(),
            "graph",
            "memberships",
            self.member_id.to_string().as_ref(),
            "up",
        ]))
    }
}

impl<'a> InvalidatesOwnCacheKey for AzureDevOpsGroupsForMemberRequest<'a> {}

impl<'a> ExecutableRequest for AzureDevOpsGroupsForMemberRequest<'a> {
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
    type Output = Arc<Vec<AzureDevOpsGraphMembership>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let organization = self.org_url.organization_name()?;
            let subject_descriptor = &self.member_id;
            let url = format!(
                "https://vssps.dev.azure.com/{organization}/_apis/graph/Memberships/{subject_descriptor}?api-version=7.1-preview.1&direction=up",
                organization = organization,
                subject_descriptor = subject_descriptor
            );
            let request = RestRequest::new(Method::GET, url.as_str())?
                .cache(self.cache_key()?)
                .authentication(self.auth_selection.into());
            let response = request
                .receive_cached::<AzureDevOpsCollectionResponse<Arc<Vec<AzureDevOpsGraphMembership>>>>(
                    &executor,
                    CacheOperationOwner::of::<AzureDevOpsGroupsForMemberRequest<'static>>(),
                )
                .await?;
            Ok(Arc::clone(&response.value))
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsGroupsForMemberRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsGroupsForMemberRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(AzureDevOpsGroupsForMemberRequest<'static> => Arc<Vec<AzureDevOpsGraphMembership>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureDevOpsGroupsForMemberRequest<'static>,
    name = "azure-devops-groups-for-member",
    cached
);
