use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsDescriptor;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsGroupMember;
use cloud_terrastodon_command::CacheBehaviour;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

pub async fn fetch_azure_devops_group_members(
    group_id: &AzureDevOpsDescriptor,
) -> eyre::Result<HashMap<AzureDevOpsDescriptor, AzureDevOpsGroupMember>> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args([
        "devops",
        "security",
        "group",
        "membership",
        "list",
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

#[cfg(test)]
mod test {
    use crate::prelude::fetch_all_azure_devops_projects;
    use crate::prelude::fetch_azure_devops_group_members;
    use crate::prelude::fetch_azure_devops_groups;
    use eyre::bail;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let projects = fetch_all_azure_devops_projects().await?;
        for project in &projects {
            let groups = fetch_azure_devops_groups(project).await?;
            for group in &groups {
                let members = fetch_azure_devops_group_members(&group.descriptor).await?;
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
}
