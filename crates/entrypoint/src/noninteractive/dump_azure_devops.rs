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
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ConcurrentIteratorExt;
use eyre::OptionExt;
use facet::Facet;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use tracing::info;

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

    let remaining = AtomicUsize::new(projects.len());
    // Preserve complete-all collection: operation errors are inspected only
    // after the scoped map has drained every admitted read.
    let project_groups = projects
        .iter()
        .map_concurrent(
            executor,
            NonZeroUsize::new(10).ok_or_eyre("project group concurrency must be nonzero")?,
            async |executor, project| {
                let result = fetch_azure_devops_groups_for_project(
                    &org_url,
                    project.id,
                    azure_devops_auth_context,
                    &executor,
                )
                .await
                .map(|groups| AzureDevOpsDumpProjectGroups {
                    project_id: project.id,
                    groups,
                });
                info!(
                    "Fetching Azure DevOps groups, {} projects remain",
                    remaining.fetch_sub(1, Ordering::Relaxed) - 1
                );
                eyre::Ok(result)
            },
        )
        .await?
        .into_iter()
        .collect::<eyre::Result<Vec<_>>>()?;

    let remaining = AtomicUsize::new(
        project_groups
            .iter()
            .map(|project| project.groups.len())
            .sum(),
    );
    let group_members = project_groups
        .iter()
        .flat_map(|project| &project.groups)
        .map_concurrent(
            executor,
            NonZeroUsize::new(10).ok_or_eyre("group membership concurrency must be nonzero")?,
            async |executor, group| {
                let result = fetch_azure_devops_group_members(
                    &org_url,
                    &group.descriptor,
                    azure_devops_auth_context,
                    &executor,
                )
                .await
                .map(|members| AzureDevOpsDumpGroupMembers {
                    group_descriptor: group.descriptor.clone(),
                    members: members.values().cloned().collect(),
                });
                info!(
                    "Fetching Azure DevOps group members, {} groups remain",
                    remaining.fetch_sub(1, Ordering::Relaxed) - 1
                );
                eyre::Ok(result)
            },
        )
        .await?
        .into_iter()
        .collect::<eyre::Result<Vec<_>>>()?;
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
