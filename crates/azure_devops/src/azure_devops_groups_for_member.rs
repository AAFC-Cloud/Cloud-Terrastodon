use arbitrary::Arbitrary;
use cloud_terrastodon_auth::AzureDevOpsAuthContext;
use cloud_terrastodon_azure_devops_types::AzureDevOpsDescriptor;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_types::ArbitraryJson;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use cloud_terrastodon_rest::RestRequest;
use facet::Facet;
use reqwest::Method;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;

#[track_caller]
pub fn fetch_azure_devops_groups_for_member<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    member_id: &'a AzureDevOpsDescriptor,
    auth_context: &'a AzureDevOpsAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, AzureDevOpsGroupsForMemberRequest<'a>> {
    executor.execute(AzureDevOpsGroupsForMemberRequest {
        org_url: Cow::Borrowed(org_url),
        member_id: Cow::Borrowed(member_id),
        auth_context: Cow::Borrowed(auth_context),
    })
}

#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsGroupsForMemberRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub member_id: Cow<'a, AzureDevOpsDescriptor>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
}

impl<'a> Arbitrary<'a> for AzureDevOpsGroupsForMemberRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            member_id: Cow::Owned(AzureDevOpsDescriptor::arbitrary(u)?),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
        })
    }
}

#[derive(Debug, Facet)]
pub struct AzureDevOpsGroupsForMemberResponse {
    pub count: usize,
    pub value: Arc<Vec<AzureDevOpsGroupsForMemberResponseEntry>>,
}
#[derive(Debug, Facet, Arbitrary)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsGroupsForMemberResponseEntry {
    pub container_descriptor: AzureDevOpsDescriptor,
    pub member_descriptor: AzureDevOpsDescriptor,
    #[facet(rename = "_links")]
    pub links: ArbitraryJson,
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
    type Error = eyre::Report;
    type Output = Arc<Vec<AzureDevOpsGroupsForMemberResponseEntry>>;
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
                .azure_devops_auth_context(self.auth_context.as_ref())?;
            let response = request
                .receive_cached::<AzureDevOpsGroupsForMemberResponse>(
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
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsGroupsForMemberResponseEntry);
cloud_terrastodon_registry_execution::register_executable_request!(AzureDevOpsGroupsForMemberRequest<'static> => Arc<Vec<AzureDevOpsGroupsForMemberResponseEntry>>, effects = [Read]);

#[cfg(test)]
mod test {
    use crate::fetch_azure_devops_groups_for_member;
    use crate::fetch_azure_devops_user_license_entitlements;
    use crate::get_default_organization_url;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_auth::AzureDevOpsAuthContext;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .with_async_scope(|cancellation| async move {
                let auth_context = AuthContext::explicit_azure_cli();
                let azure_devops_auth_context = AzureDevOpsAuthContext::for_tenant(
                    &auth_context,
                    crate::test_tenant_id::test_tenant_id()?,
                )?;
                let executor =
                    crate::test_executor::test_executor(&azure_devops_auth_context, &cancellation)?;
                let org_url = get_default_organization_url(&cancellation).await?;
                let users = fetch_azure_devops_user_license_entitlements(
                    &org_url,
                    &azure_devops_auth_context,
                    &executor,
                )
                .await?;
                for user in users.iter().take(2) {
                    let groups_for_user = fetch_azure_devops_groups_for_member(
                        &org_url,
                        &user.user.descriptor,
                        &azure_devops_auth_context,
                        &executor,
                    )
                    .await?;
                    assert!(
                        groups_for_user
                            .iter()
                            .all(|entry| entry.member_descriptor == user.user.descriptor)
                    );
                }
                Ok(())
            })
            .await
    }
}
