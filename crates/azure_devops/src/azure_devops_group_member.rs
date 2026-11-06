use arbitrary::Arbitrary;
use cloud_terrastodon_auth::AzureDevOpsAuthContext;
use cloud_terrastodon_azure_devops_types::AzureDevOpsCollectionResponse;
use cloud_terrastodon_azure_devops_types::AzureDevOpsDescriptor;
use cloud_terrastodon_azure_devops_types::AzureDevOpsGraphMembership;
use cloud_terrastodon_azure_devops_types::AzureDevOpsGraphSubjectLookupBody;
use cloud_terrastodon_azure_devops_types::AzureDevOpsGraphSubjectLookupKey;
use cloud_terrastodon_azure_devops_types::AzureDevOpsGroupMember;
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
use reqwest::Method;
use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, facet::Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsGroupMembersListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
    pub group_id: Cow<'a, AzureDevOpsDescriptor>,
}

#[track_caller]
pub fn fetch_azure_devops_group_members<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    group_id: &'a AzureDevOpsDescriptor,
    auth_context: &'a AzureDevOpsAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, AzureDevOpsGroupMembersListRequest<'a>> {
    executor.execute(AzureDevOpsGroupMembersListRequest {
        org_url: Cow::Borrowed(org_url),
        auth_context: Cow::Borrowed(auth_context),
        group_id: Cow::Borrowed(group_id),
    })
}

impl<'a> Arbitrary<'a> for AzureDevOpsGroupMembersListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
            group_id: Cow::Owned(AzureDevOpsDescriptor::arbitrary(u)?),
        })
    }
}
impl<'a> HasCacheKey for AzureDevOpsGroupMembersListRequest<'a> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "devops",
            self.org_url.name(),
            "security",
            "group",
            "membership",
            "list",
            "--id",
            self.group_id.to_string().as_ref(),
        ]))
    }
}

impl<'a> InvalidatesOwnCacheKey for AzureDevOpsGroupMembersListRequest<'a> {}

impl<'a> ExecutableRequest for AzureDevOpsGroupMembersListRequest<'a> {
    type Error = eyre::Report;
    type Output = Arc<HashMap<AzureDevOpsDescriptor, Arc<AzureDevOpsGroupMember>>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            // Membership edges identify descriptors; subject lookup supplies the
            // records returned by the former CLI projection.
            // https://learn.microsoft.com/en-us/rest/api/azure/devops/graph/memberships/list?view=azure-devops-rest-7.1
            let cache_key = self.cache_key()?;
            let mut url = self.org_url.api_url(
                "vssps.dev.azure.com",
                "",
                &[
                    ("api-version", "7.1-preview.1"),
                    ("direction", "down"),
                    ("depth", "1"),
                ],
            )?;
            url.path_segments_mut()
                .map_err(|()| eyre::eyre!("organization URL must have a hierarchical path"))?
                .pop_if_empty()
                .extend(["_apis", "graph", "memberships", &self.group_id.to_string()]);
            let memberships = RestRequest::from_method_and_url(Method::GET, url)?
                .cache(cache_key.join("memberships")?)
                .azure_devops_auth_context(self.auth_context.as_ref())?
                .receive_cached::<AzureDevOpsCollectionResponse<Vec<AzureDevOpsGraphMembership>>>(
                    &executor,
                    CacheOperationOwner::of::<AzureDevOpsGroupMembersListRequest<'static>>(),
                )
                .await?;
            executor.cancellation().bail_if_cancelled()?;
            if memberships.value.is_empty() {
                return Ok(Arc::new(HashMap::new()));
            }
            let body = AzureDevOpsGraphSubjectLookupBody {
                lookup_keys: memberships
                    .value
                    .iter()
                    .map(|membership| AzureDevOpsGraphSubjectLookupKey {
                        descriptor: membership.member_descriptor.clone(),
                    })
                    .collect(),
            };
            // This documented POST only resolves subjects; it does not mutate them.
            // https://learn.microsoft.com/en-us/rest/api/azure/devops/graph/subject-lookup/lookup-subjects?view=azure-devops-rest-7.1
            let url = self.org_url.api_url(
                "vssps.dev.azure.com",
                "_apis/graph/subjectlookup",
                &[("api-version", "7.1-preview.1")],
            )?;
            let response = RestRequest::from_method_and_url(Method::POST, url)?
                .cancellation_safe()
                .body(facet_json::to_string(&body)?)
                .cache(cache_key.join("subjects")?)
                .azure_devops_auth_context(self.auth_context.as_ref())?
                .receive_cached::<AzureDevOpsCollectionResponse<
                    Arc<HashMap<AzureDevOpsDescriptor, Arc<AzureDevOpsGroupMember>>>,
                >>(
                    &executor,
                    CacheOperationOwner::of::<AzureDevOpsGroupMembersListRequest<'static>>(),
                )
                .await?;
            Ok(Arc::clone(&response.value))
        })
    }
}

#[derive(Debug, PartialEq, Eq, facet::Facet)]
#[facet(transparent)]
pub struct AzureDevOpsGroupMembersV2Response(pub ArbitraryJson);

impl<'a> Arbitrary<'a> for AzureDevOpsGroupMembersV2Response {
    fn arbitrary(_u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        // Keep the response's object envelope when generating offline values.
        Ok(Self(ArbitraryJson::object()))
    }
}

impl Deref for AzureDevOpsGroupMembersV2Response {
    type Target = ArbitraryJson;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, facet::Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsGroupMembersV2Request<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub group_id: Cow<'a, AzureDevOpsDescriptor>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
}

#[track_caller]
pub fn fetch_azure_devops_group_members_v2<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    group_id: &'a AzureDevOpsDescriptor,
    auth_context: &'a AzureDevOpsAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, AzureDevOpsGroupMembersV2Request<'a>> {
    executor.execute(AzureDevOpsGroupMembersV2Request {
        org_url: Cow::Borrowed(org_url),
        group_id: Cow::Borrowed(group_id),
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl<'a> Arbitrary<'a> for AzureDevOpsGroupMembersV2Request<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            group_id: Cow::Owned(AzureDevOpsDescriptor::arbitrary(u)?),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
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
                .azure_devops_auth_context(self.auth_context.as_ref())?
                .receive_cached::<AzureDevOpsGroupMembersV2Response>(
                    &executor,
                    CacheOperationOwner::of::<AzureDevOpsGroupMembersV2Request<'static>>(),
                )
                .await?;
            Ok(response)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsGroupMembersListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsGroupMembersListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureDevOpsGroupMembersListRequest<'static> => Arc<HashMap<AzureDevOpsDescriptor, Arc<AzureDevOpsGroupMember>>>,
    effects = [Read]
);
cloud_terrastodon_registry::register_thing!(AzureDevOpsGroupMembersV2Response);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsGroupMembersV2Response);
cloud_terrastodon_registry::register_arbitrary!(Arc<AzureDevOpsGroupMembersV2Response>);
cloud_terrastodon_registry::register_thing!(AzureDevOpsGroupMembersV2Request<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsGroupMembersV2Request<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureDevOpsGroupMembersV2Request<'static> => Arc<AzureDevOpsGroupMembersV2Response>,
    effects = [Read]
);

#[cfg(test)]
mod test {
    use crate::fetch_all_azure_devops_projects;
    use crate::fetch_azure_devops_group_members;
    use crate::fetch_azure_devops_group_members_v2;
    use crate::fetch_azure_devops_groups_for_project;
    use crate::get_default_organization_url;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_auth::AzureDevOpsAuthContext;
    use cloud_terrastodon_azure_devops_types::AzureDevOpsDescriptor;
    use cloud_terrastodon_azure_devops_types::AzureDevOpsGroupMember;
    use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
    use cloud_terrastodon_azure_types::ArbitraryJson;
    use cloud_terrastodon_command::CancellationToken;
    use eyre::bail;
    use std::collections::HashMap;
    use std::str::FromStr;

    #[test]
    pub fn parses_group_members_with_typed_descriptor_map_keys() -> eyre::Result<()> {
        let json = r#"{
            "aad.user": {
                "description": null,
                "descriptor": "aad.user",
                "displayName": "Ada",
                "domain": "aad",
                "legacyDescriptor": null,
                "mailAddress": null,
                "origin": "aad",
                "originId": "origin-id",
                "principalName": "ada@example.com",
                "subjectKind": "user",
                "url": "https://example.invalid"
            }
        }"#;

        let members =
            facet_json::from_str::<HashMap<AzureDevOpsDescriptor, AzureDevOpsGroupMember>>(json)?;

        let descriptor: AzureDevOpsDescriptor = "aad.user".parse()?;
        let member = members
            .get(&descriptor)
            .ok_or_else(|| eyre::eyre!("missing parsed descriptor key"))?;
        assert_eq!(member.display_name, "Ada");
        assert_eq!(member.descriptor, descriptor);
        Ok(())
    }

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .with_async_scope(|cancellation| async move {
                let auth_context = AuthContext::explicit_azure_cli();
                let azure_devops_auth_context = AzureDevOpsAuthContext::for_tenant(&auth_context, crate::test_tenant_id::test_tenant_id()?)?;
                let executor = crate::test_executor::test_executor(&azure_devops_auth_context, &cancellation)?;
                let org_url = get_default_organization_url(&cancellation).await?;
                let projects =
                    fetch_all_azure_devops_projects(&org_url, &azure_devops_auth_context, &executor)
                        .await?;
                for project in &projects {
                    let groups = fetch_azure_devops_groups_for_project(
                        &org_url,
                        &project.name,
                        &azure_devops_auth_context,
                        &executor,
                    )
                    .await?;
                    for group in &groups {
                        let members =
                            fetch_azure_devops_group_members(&org_url, &group.descriptor, &azure_devops_auth_context, &executor)
                                .await?;
                        if !members.is_empty() {
                            assert!(
                                members.iter().all(|(descriptor, member)| {
                                    descriptor == &member.descriptor && !member.display_name.is_empty()
                                }),
                                "Expected Azure DevOps group members to include matching descriptors and display names"
                            );
                            return Ok(());
                        }
                    }
                }
                bail!("No Azure DevOps group with members found in any project");

            })
            .await
    }

    #[tokio::test]
    #[ignore]
    pub async fn it_works_v2() -> eyre::Result<()> {
        CancellationToken::new()
            .with_async_scope(|cancellation| async move {
                #[derive(facet::Facet)]
                struct MembershipsResponse {
                    #[facet(default)]
                    value: Option<ArbitraryJson>,
                    #[facet(default)]
                    count: Option<usize>,
                }

                let org = AzureDevOpsOrganizationUrl::from_str("https://dev.azure.com/aafc/")?;
                let desc = AzureDevOpsDescriptor::AzureDevOpsGroup("vssgp.redacted".to_string());
                let auth_context = AuthContext::explicit_azure_cli();
                let azure_devops_auth_context = AzureDevOpsAuthContext::for_tenant(
                    &auth_context,
                    crate::test_tenant_id::test_tenant_id()?,
                )?;
                let executor =
                    crate::test_executor::test_executor(&azure_devops_auth_context, &cancellation)?;
                let resp = fetch_azure_devops_group_members_v2(
                    &org,
                    &desc,
                    &azure_devops_auth_context,
                    &executor,
                )
                .await?;
                let resp: MembershipsResponse = facet_json::from_str(resp.as_str())?;
                assert!(
                    resp.value.is_some() || resp.count.is_some(),
                    "Expected Azure DevOps group memberships V2 response payload"
                );
                Ok(())
            })
            .await
    }
}
