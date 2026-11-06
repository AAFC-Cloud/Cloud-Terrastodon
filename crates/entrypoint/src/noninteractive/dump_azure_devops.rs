use cloud_terrastodon_auth::AzureDevOpsAuthContext;
use cloud_terrastodon_azure_devops::AzureDevOpsDescriptor;
use cloud_terrastodon_azure_devops::AzureDevOpsGroup;
use cloud_terrastodon_azure_devops::AzureDevOpsGroupMember;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProject;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectId;
use cloud_terrastodon_azure_devops::AzureDevOpsUserLicenseEntitlement;
use cloud_terrastodon_azure_devops::fetch_all_azure_devops_projects;
use cloud_terrastodon_azure_devops::fetch_azure_devops_group_members;
use cloud_terrastodon_azure_devops::fetch_azure_devops_groups_for_project;
use cloud_terrastodon_azure_devops::fetch_azure_devops_user_license_entitlements;
use cloud_terrastodon_command::ParallelFallibleWorkQueue;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use facet::Facet;
use std::sync::Arc;

#[derive(Facet)]
struct AzureDevOpsDumpPayload {
    projects: Arc<Vec<Arc<AzureDevOpsProject>>>,
    users: Vec<Arc<AzureDevOpsUserLicenseEntitlement>>,
    project_groups: Vec<AzureDevOpsDumpProjectGroups>,
    group_members: Vec<AzureDevOpsDumpGroupMembers>,
}

#[derive(Facet)]
struct AzureDevOpsDumpProjectGroups {
    project_id: AzureDevOpsProjectId,
    groups: Vec<Arc<AzureDevOpsGroup>>,
}

#[derive(Facet)]
struct AzureDevOpsDumpGroupMembers {
    group_descriptor: AzureDevOpsDescriptor,
    members: Vec<Arc<AzureDevOpsGroupMember>>,
}

/// Write to stdout the json for a bunch of Azure DevOps info
pub async fn dump_azure_devops(
    org_url: AzureDevOpsOrganizationUrl,
    azure_devops_auth_context: &AzureDevOpsAuthContext,
    executor: &CloudTerrastodonExecutor,
) -> eyre::Result<()> {
    let projects =
        fetch_all_azure_devops_projects(&org_url, azure_devops_auth_context.selection()?, executor)
            .await?;

    let users =
        fetch_azure_devops_user_license_entitlements(&org_url, azure_devops_auth_context, executor)
            .await?;

    let mut project_groups = ParallelFallibleWorkQueue::new("fetch_azure_devops_groups", 10);
    for project in projects.iter() {
        let org_url = org_url.clone();
        let project_id = project.id.clone();
        let azure_devops_auth_context = azure_devops_auth_context.clone();
        project_groups.enqueue({
            executor.cancellable_scope(move |executor| async move {
                let executor = &executor;
                let groups = fetch_azure_devops_groups_for_project(
                    &org_url,
                    &project_id,
                    &azure_devops_auth_context,
                    executor,
                )
                .await?;
                eyre::Ok((project_id, groups))
            })
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
        let azure_devops_auth_context = azure_devops_auth_context.clone();
        group_members.enqueue({
            executor.cancellable_scope(move |executor| async move {
                let executor = &executor;
                let members = fetch_azure_devops_group_members(
                    &org_url,
                    &group.descriptor,
                    &azure_devops_auth_context,
                    executor,
                )
                .await?;
                eyre::Ok(AzureDevOpsDumpGroupMembers {
                    group_descriptor: group.descriptor.clone(),
                    members: members.values().cloned().collect(),
                })
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
        facet_json::to_string_pretty(&payload).map_err(|error| eyre::eyre!("{error:?}"))?
    );
    Ok(())
}
