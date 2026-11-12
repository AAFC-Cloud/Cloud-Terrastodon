use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops_types::AzureDevOpsCollectionResponse;
use cloud_terrastodon_azure_devops_types::AzureDevOpsDescriptor;
use cloud_terrastodon_azure_devops_types::AzureDevOpsGraphMembership;
use cloud_terrastodon_azure_devops_types::AzureDevOpsGraphSubjectLookupBody;
use cloud_terrastodon_azure_devops_types::AzureDevOpsGraphSubjectLookupKey;
use cloud_terrastodon_azure_devops_types::AzureDevOpsGroupMember;
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
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsGroupMembersListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub auth_selection: AzureDevOpsAuthSelection,
    pub group_id: Cow<'a, AzureDevOpsDescriptor>,
}

impl<'a> Arbitrary<'a> for AzureDevOpsGroupMembersListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            auth_selection: AzureDevOpsAuthSelection::arbitrary(u)?,
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
                .authentication(self.auth_selection.into())
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
                .authentication(self.auth_selection.into())
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

cloud_terrastodon_registry::register_thing!(AzureDevOpsGroupMembersListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsGroupMembersListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureDevOpsGroupMembersListRequest<'static> => Arc<HashMap<AzureDevOpsDescriptor, Arc<AzureDevOpsGroupMember>>>,
    effects = [Read]
);

#[cfg(test)]
mod test {

    use cloud_terrastodon_azure_devops_types::AzureDevOpsDescriptor;
    use cloud_terrastodon_azure_devops_types::AzureDevOpsGroupMember;
    use std::collections::HashMap;

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
}

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureDevOpsGroupMembersListRequest<'static>,
    name = "azure-devops-group-members-list",
    cached
);
