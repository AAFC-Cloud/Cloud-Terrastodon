use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsDescriptor;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use serde::Deserialize;
use std::path::PathBuf;

pub fn fetch_azure_devops_groups_for_member<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    member_id: &'a AzureDevOpsDescriptor,
) -> AzureDevOpsGroupsForMemberRequest<'a> {
    AzureDevOpsGroupsForMemberRequest { org_url, member_id }
}

pub struct AzureDevOpsGroupsForMemberRequest<'a> {
    pub org_url: &'a AzureDevOpsOrganizationUrl,
    pub member_id: &'a AzureDevOpsDescriptor,
}

#[derive(Debug, Deserialize)]
pub struct AzureDevOpsGroupsForMemberResponse {
    pub count: usize,
    pub value: Vec<AzureDevOpsGroupsForMemberResponseEntry>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AzureDevOpsGroupsForMemberResponseEntry {
    pub container_descriptor: AzureDevOpsDescriptor,
    pub member_descriptor: AzureDevOpsDescriptor,
    #[serde(rename = "_links")]
    pub _links: serde_json::Value,
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
        let subject_descriptor = self.member_id;
        let url = format!(
            "https://vssps.dev.azure.com/{organization}/_apis/graph/Memberships/{subject_descriptor}?api-version=7.1-preview.1&direction=up",
            organization = organization,
            subject_descriptor = subject_descriptor
        );
        let mut cmd = CommandBuilder::new(CommandKind::CloudTerrastodon);
        cmd.cache(self.cache_key());
        cmd.args([
            "az",
            "devops",
            "rest",
            "--method",
            "GET",
            "--url",
            url.as_ref(),
        ]);
        Ok(cmd.run::<AzureDevOpsGroupsForMemberResponse>().await?.value)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(AzureDevOpsGroupsForMemberRequest<'a>, 'a);

#[cfg(test)]
mod test {
    use crate::prelude::fetch_azure_devops_groups_for_member;
    use crate::prelude::fetch_azure_devops_license_entitlements;
    use crate::prelude::get_default_organization_url;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let org_url = get_default_organization_url().await?;
        let users = fetch_azure_devops_license_entitlements(&org_url).await?;
        for user in users.iter().take(2) {
            let groups_for_user =
                fetch_azure_devops_groups_for_member(&org_url, &user.user.descriptor).await?;
            println!("{:#?}", groups_for_user);
        }
        Ok(())
    }
}
