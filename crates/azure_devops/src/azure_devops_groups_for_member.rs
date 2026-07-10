use cloud_terrastodon_azure_devops_types::AzureDevOpsDescriptor;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_types::ArbitraryJson;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_rest::RestRequest;
use reqwest::Method;
use std::borrow::Cow;
use std::path::PathBuf;

pub fn fetch_azure_devops_groups_for_member<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    member_id: &'a AzureDevOpsDescriptor,
) -> AzureDevOpsGroupsForMemberRequest<'a> {
    AzureDevOpsGroupsForMemberRequest {
        org_url: Cow::Borrowed(org_url),
        member_id: Cow::Borrowed(member_id),
    }
}

#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsGroupsForMemberRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub member_id: Cow<'a, AzureDevOpsDescriptor>,
}

impl<'a> Arbitrary<'a> for AzureDevOpsGroupsForMemberRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            member_id: Cow::Owned(AzureDevOpsDescriptor::arbitrary(u)?),
        })
    }
}

#[derive(Debug, facet::Facet)]
pub struct AzureDevOpsGroupsForMemberResponse {
    pub count: usize,
    pub value: Vec<AzureDevOpsGroupsForMemberResponseEntry>,
}
#[derive(Debug, facet::Facet, Arbitrary)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsGroupsForMemberResponseEntry {
    pub container_descriptor: AzureDevOpsDescriptor,
    pub member_descriptor: AzureDevOpsDescriptor,
    #[facet(rename = "_links")]
    pub links: ArbitraryJson,
}

#[async_trait]
impl<'a> cloud_terrastodon_command::CacheableCommand for AzureDevOpsGroupsForMemberRequest<'a> {
    type Output = Vec<AzureDevOpsGroupsForMemberResponseEntry>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "devops",
            self.org_url.organization_name.as_ref(),
            "graph",
            "memberships",
            self.member_id.to_string().as_ref(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        let organization = &self.org_url.organization_name;
        let subject_descriptor = &self.member_id;
        let url = format!(
            "https://vssps.dev.azure.com/{organization}/_apis/graph/Memberships/{subject_descriptor}?api-version=7.1-preview.1&direction=up",
            organization = organization,
            subject_descriptor = subject_descriptor
        );
        Ok(RestRequest::new(Method::GET, url.as_str())?
            .cache(self.cache_key())
            .receive::<AzureDevOpsGroupsForMemberResponse>()
            .await?
            .value)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(AzureDevOpsGroupsForMemberRequest<'a>, 'a);
cloud_terrastodon_registry::register_thing!(AzureDevOpsGroupsForMemberRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsGroupsForMemberRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(Vec<AzureDevOpsGroupsForMemberResponseEntry>);
cloud_terrastodon_registry::register_into_future!(AzureDevOpsGroupsForMemberRequest<'static> => Vec<AzureDevOpsGroupsForMemberResponseEntry>, effects = [Read]);

#[cfg(test)]
mod test {
    use crate::fetch_azure_devops_groups_for_member;
    use crate::fetch_azure_devops_user_license_entitlements;
    use crate::get_default_organization_url;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let org_url = get_default_organization_url().await?;
        let users = fetch_azure_devops_user_license_entitlements(&org_url).await?;
        for user in users.iter().take(2) {
            let groups_for_user =
                fetch_azure_devops_groups_for_member(&org_url, &user.user.descriptor).await?;
            assert!(
                groups_for_user
                    .iter()
                    .all(|entry| entry.member_descriptor == user.user.descriptor)
            );
        }
        Ok(())
    }
}
use arbitrary::Arbitrary;
