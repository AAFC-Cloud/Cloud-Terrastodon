use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsDescriptor;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsGroupMember;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::create_azure_devops_rest_client;
use cloud_terrastodon_credentials::get_azure_devops_personal_access_token_from_credential_manager;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct AzureDevOpsGroupMembersListRequest<'a> {
    pub org_url: &'a AzureDevOpsOrganizationUrl,
    pub group_id: &'a AzureDevOpsDescriptor,
}

pub fn fetch_azure_devops_group_members<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    group_id: &'a AzureDevOpsDescriptor,
) -> AzureDevOpsGroupMembersListRequest<'a> {
    AzureDevOpsGroupMembersListRequest { org_url, group_id }
}

#[async_trait]
impl<'a> cloud_terrastodon_command::CacheableCommand for AzureDevOpsGroupMembersListRequest<'a> {
    type Output = HashMap<AzureDevOpsDescriptor, AzureDevOpsGroupMember>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "devops",
            self.org_url.organization_name.as_ref(),
            "security",
            "group",
            "membership",
            "list",
            "--id",
            self.group_id.to_string().as_ref(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        let org = self.org_url.to_string();
        let gid = self.group_id.to_string();
        cmd.args([
            "devops",
            "security",
            "group",
            "membership",
            "list",
            "--organization",
            org.as_str(),
            "--id",
            gid.as_str(),
            "--output",
            "json",
        ]);
        cmd.cache(self.cache_key());
        cmd.run().await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(AzureDevOpsGroupMembersListRequest<'a>, 'a);

pub struct AzureDevOpsGroupMembersV2Request<'a> {
    pub org_url: &'a AzureDevOpsOrganizationUrl,
    pub group_id: &'a AzureDevOpsDescriptor,
}

pub fn fetch_azure_devops_group_members_v2<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    group_id: &'a AzureDevOpsDescriptor,
) -> AzureDevOpsGroupMembersV2Request<'a> {
    AzureDevOpsGroupMembersV2Request { org_url, group_id }
}

#[async_trait]
impl<'a> cloud_terrastodon_command::CacheableCommand for AzureDevOpsGroupMembersV2Request<'a> {
    type Output = serde_json::Value;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "devops",
            self.org_url.organization_name.as_ref(),
            "graph",
            "memberships",
            self.group_id.to_string().as_ref(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        let organization = &self.org_url.organization_name;
        let subject_descriptor = self.group_id;
        let url = format!(
            "https://vssps.dev.azure.com/{organization}/_apis/graph/Memberships/{subject_descriptor}?api-version=7.1-preview.1&direction=down",
            organization = organization,
            subject_descriptor = subject_descriptor
        );
        let client = create_azure_devops_rest_client(
            &get_azure_devops_personal_access_token_from_credential_manager().await?,
        )
        .await?;
        let resp = client.get(url).send().await?;
        let resp = resp.json().await?;
        Ok(resp)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(AzureDevOpsGroupMembersV2Request<'a>, 'a);

#[cfg(test)]
mod test {
    use crate::prelude::fetch_all_azure_devops_projects;
    use crate::prelude::fetch_azure_devops_group_members;
    use crate::prelude::fetch_azure_devops_group_members_v2;
    use crate::prelude::fetch_azure_devops_groups_for_project;
    use crate::prelude::get_default_organization_url;
    use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsDescriptor;
    use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsOrganizationUrl;
    use eyre::bail;
    use std::str::FromStr;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let org_url = get_default_organization_url().await?;
        let projects = fetch_all_azure_devops_projects(&org_url).await?;
        for project in &projects {
            let groups = fetch_azure_devops_groups_for_project(&org_url, &project.name).await?;
            for group in &groups {
                let members = fetch_azure_devops_group_members(&org_url, &group.descriptor).await?;
                if !members.is_empty() {
                    println!(
                        "Found group with members in project '{}': group '{}'",
                        project.name, group.display_name
                    );
                    for (descriptor, member) in members {
                        println!("Member: {} - {}", descriptor, member.display_name);
                    }
                    return Ok(());
                }
            }
        }
        bail!("No Azure DevOps group with members found in any project");
    }

    #[tokio::test]
    #[ignore]
    pub async fn it_works_v2() -> eyre::Result<()> {
        let org = AzureDevOpsOrganizationUrl::from_str("https://dev.azure.com/aafc/")?;
        let desc = AzureDevOpsDescriptor::AzureDevOpsGroup("vssgp.redacted".to_string());
        let resp = fetch_azure_devops_group_members_v2(&org, &desc).await?;
        println!("{resp:#?}");
        Ok(())
    }
}
