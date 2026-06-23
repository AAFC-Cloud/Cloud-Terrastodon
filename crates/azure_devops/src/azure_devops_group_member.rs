use cloud_terrastodon_azure_devops_types::AzureDevOpsDescriptor;
use cloud_terrastodon_azure_devops_types::AzureDevOpsGroupMember;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::create_azure_devops_rest_client;
use cloud_terrastodon_credentials::get_azure_devops_personal_access_token_from_credential_manager;
use eyre::Context;
use facet_json::RawJson;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

fn parse_group_members_by_descriptor(
    members_by_descriptor: HashMap<String, AzureDevOpsGroupMember>,
) -> eyre::Result<HashMap<AzureDevOpsDescriptor, AzureDevOpsGroupMember>> {
    members_by_descriptor
        .into_iter()
        .map(|(descriptor, member)| {
            Ok((
                AzureDevOpsDescriptor::from_str(&descriptor).wrap_err_with(|| {
                    format!("parsing Azure DevOps descriptor map key {descriptor:?}")
                })?,
                member,
            ))
        })
        .collect()
}

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
        cmd.run_with_mapper(parse_group_members_by_descriptor).await
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
    type Output = RawJson<'static>;

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
        Ok(RawJson::from_owned(resp.text().await?))
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(AzureDevOpsGroupMembersV2Request<'a>, 'a);

#[cfg(test)]
mod test {
    use crate::fetch_all_azure_devops_projects;
    use crate::fetch_azure_devops_group_members;
    use crate::fetch_azure_devops_group_members_v2;
    use crate::fetch_azure_devops_groups_for_project;
    use crate::get_default_organization_url;
    use cloud_terrastodon_azure_devops_types::AzureDevOpsDescriptor;
    use cloud_terrastodon_azure_devops_types::AzureDevOpsGroupMember;
    use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
    use eyre::bail;
    use facet_json::RawJson;
    use std::collections::HashMap;
    use std::str::FromStr;

    #[test]
    pub fn parses_group_members_with_descriptor_map_keys() -> eyre::Result<()> {
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

        let members_by_descriptor =
            facet_json::from_str::<HashMap<String, AzureDevOpsGroupMember>>(json)?;
        let members = super::parse_group_members_by_descriptor(members_by_descriptor)?;

        let descriptor = AzureDevOpsDescriptor::from_str("aad.user")?;
        let member = members
            .get(&descriptor)
            .ok_or_else(|| eyre::eyre!("missing parsed descriptor key"))?;
        assert_eq!(member.display_name, "Ada");
        assert_eq!(member.descriptor, descriptor);
        Ok(())
    }

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let org_url = get_default_organization_url().await?;
        let projects = fetch_all_azure_devops_projects(&org_url).await?;
        for project in &projects {
            let groups = fetch_azure_devops_groups_for_project(&org_url, &project.name).await?;
            for group in &groups {
                let members = fetch_azure_devops_group_members(&org_url, &group.descriptor).await?;
                if !members.is_empty() {
                    assert!(
                        members.into_iter().all(|(descriptor, member)| {
                            descriptor == member.descriptor && !member.display_name.is_empty()
                        }),
                        "Expected Azure DevOps group members to include matching descriptors and display names"
                    );
                    return Ok(());
                }
            }
        }
        bail!("No Azure DevOps group with members found in any project");
    }

    #[tokio::test]
    #[ignore]
    pub async fn it_works_v2() -> eyre::Result<()> {
        #[derive(facet::Facet)]
        struct MembershipsResponse {
            #[facet(default)]
            value: Option<RawJson<'static>>,
            #[facet(default)]
            count: Option<usize>,
        }

        let org = AzureDevOpsOrganizationUrl::from_str("https://dev.azure.com/aafc/")?;
        let desc = AzureDevOpsDescriptor::AzureDevOpsGroup("vssgp.redacted".to_string());
        let resp = fetch_azure_devops_group_members_v2(&org, &desc).await?;
        let resp: MembershipsResponse = facet_json::from_str(resp.as_str())?;
        assert!(
            resp.value.is_some() || resp.count.is_some(),
            "Expected Azure DevOps group memberships V2 response payload"
        );
        Ok(())
    }
}
