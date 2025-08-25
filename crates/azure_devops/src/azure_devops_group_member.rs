use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsDescriptor;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsGroupMember;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_command::CacheBehaviour;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_credentials::create_azure_devops_rest_client;
use cloud_terrastodon_credentials::get_azure_devops_pat;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

pub async fn fetch_azure_devops_group_members(
    org_url: &AzureDevOpsOrganizationUrl,
    group_id: &AzureDevOpsDescriptor,
) -> eyre::Result<HashMap<AzureDevOpsDescriptor, AzureDevOpsGroupMember>> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args([
        "devops",
        "security",
        "group",
        "membership",
        "list",
        "--organization",
        org_url.to_string().as_str(),
        "--id",
        &group_id.to_string(),
        "--output",
        "json",
    ]);
    cmd.use_cache_behaviour(CacheBehaviour::Some {
        path: PathBuf::from_iter([
            "az",
            "devops",
            "security",
            "group",
            "membership",
            "list",
            "--id",
            &group_id.to_string(),
        ]),
        valid_for: Duration::from_hours(8),
    });
    cmd.run().await
}

pub async fn fetch_azure_devops_group_members_v2(
    org_url: &AzureDevOpsOrganizationUrl,
    group_id: &AzureDevOpsDescriptor,
) -> eyre::Result<serde_json::Value> {
    // az devops invoke --area Graph --resource MembershipsBatch --query-parameters "direction=down" --route-parameters subjectDescriptor=vssgp.redacted --api-version 7.2-preview
    let organization = &org_url.organization_name;
    let subject_descriptor = &group_id;
    let url = format!(
        "https://vssps.dev.azure.com/{organization}/_apis/graph/Memberships/{subject_descriptor}?api-version=7.1-preview.1&direction=down"
    );
    let client = create_azure_devops_rest_client(&get_azure_devops_pat().await?).await?;
    let resp = client.get(url).send().await?;
    let resp = resp.json().await?;
    Ok(resp)
}

pub async fn fetch_azure_devops_groups_for_member(
    org_url: &AzureDevOpsOrganizationUrl,
    member_id: &AzureDevOpsDescriptor,
) -> eyre::Result<serde_json::Value> {
    let organization = &org_url.organization_name;
    let subject_descriptor = &member_id;
    let url = format!(
        "https://vssps.dev.azure.com/{organization}/_apis/graph/Memberships/{subject_descriptor}?api-version=7.1-preview.1&direction=up"
    );
    let client = create_azure_devops_rest_client(&get_azure_devops_pat().await?).await?;
    let resp = client.get(url).send().await?;
    let resp = resp.json().await?;
    Ok(resp)
}

#[cfg(test)]
mod test {
    use crate::prelude::fetch_all_azure_devops_projects;
    use crate::prelude::fetch_azure_devops_group_members;
    use crate::prelude::fetch_azure_devops_group_members_v2;
    use crate::prelude::fetch_azure_devops_groups;
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
            let groups = fetch_azure_devops_groups(&org_url, project).await?;
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
        let resp = fetch_azure_devops_group_members_v2(
            &AzureDevOpsOrganizationUrl::from_str("https://dev.azure.com/aafc/")?,
            &AzureDevOpsDescriptor::AzureDevOpsGroup("vssgp.redacted".to_string()),
        )
        .await?;
        println!("{resp:#?}");
        Ok(())
    }
}
