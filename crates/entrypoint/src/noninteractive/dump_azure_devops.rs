use cloud_terrastodon_azure_devops::fetch_all_azure_devops_projects;
use cloud_terrastodon_azure_devops::fetch_azure_devops_group_members;
use cloud_terrastodon_azure_devops::fetch_azure_devops_groups_for_project;
use cloud_terrastodon_azure_devops::fetch_azure_devops_user_license_entitlements;
use cloud_terrastodon_azure_devops::get_default_organization_url;
use cloud_terrastodon_command::ParallelFallibleWorkQueue;

#[derive(facet::Facet)]
struct AzureDevOpsDumpPayload {
    projects: Vec<cloud_terrastodon_azure_devops::AzureDevOpsProject>,
    users: Vec<cloud_terrastodon_azure_devops::AzureDevOpsUserLicenseEntitlement>,
    project_groups: Vec<AzureDevOpsDumpProjectGroups>,
    group_members: Vec<AzureDevOpsDumpGroupMembers>,
}

#[derive(facet::Facet)]
struct AzureDevOpsDumpProjectGroups {
    project_id: cloud_terrastodon_azure_devops::AzureDevOpsProjectId,
    groups: Vec<cloud_terrastodon_azure_devops::AzureDevOpsGroup>,
}

#[derive(facet::Facet)]
struct AzureDevOpsDumpGroupMembers {
    group_descriptor: cloud_terrastodon_azure_devops::AzureDevOpsDescriptor,
    members: Vec<cloud_terrastodon_azure_devops::AzureDevOpsGroupMember>,
}

/// Write to stdout the json for a bunch of Azure DevOps info
pub async fn dump_azure_devops() -> eyre::Result<()> {
    let org_url = get_default_organization_url().await?;
    let projects = fetch_all_azure_devops_projects(&org_url).await?;

    let users = fetch_azure_devops_user_license_entitlements(&org_url).await?;

    let mut project_groups = ParallelFallibleWorkQueue::new("fetch_azure_devops_groups", 10);
    for project in projects.iter() {
        let org_url = org_url.clone();
        let project_id = project.id.clone();
        project_groups.enqueue(async move {
            let groups = fetch_azure_devops_groups_for_project(&org_url, &project_id).await?;
            eyre::Ok((project_id, groups))
        });
    }

    let project_groups = project_groups
        .join()
        .await?
        .into_iter()
        .map(|(project_id, groups)| AzureDevOpsDumpProjectGroups { project_id, groups })
        .collect::<Vec<_>>();

    let mut group_members = ParallelFallibleWorkQueue::new("group_members", 10);
    for group in project_groups
        .iter()
        .flat_map(|project_group| project_group.groups.iter())
        .cloned()
    {
        let org_url = org_url.clone();
        group_members.enqueue(async move {
            let members = fetch_azure_devops_group_members(&org_url, &group.descriptor).await?;
            eyre::Ok(AzureDevOpsDumpGroupMembers {
                group_descriptor: group.descriptor.clone(),
                members: members.into_values().collect(),
            })
        });
    }
    let group_members = group_members.join().await?;

    let payload = AzureDevOpsDumpPayload {
        projects,
        users,
        project_groups,
        group_members,
    };

    println!(
        "{}",
        cloud_terrastodon_command::to_string_pretty(&payload)?
    );
    Ok(())
}
