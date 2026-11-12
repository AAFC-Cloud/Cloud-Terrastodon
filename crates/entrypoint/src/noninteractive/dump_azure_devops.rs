use super::azure_devops_dump_group_members::AzureDevOpsDumpGroupMembers;
use super::azure_devops_dump_payload::AzureDevOpsDumpPayload;
use super::azure_devops_dump_project_groups::AzureDevOpsDumpProjectGroups;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops::AzureDevOpsGroupMembersListRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsGroupsListRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectsListRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsUserLicenseEntitlementListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ConcurrentIteratorExt;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::OptionExt;
use std::borrow::Cow;
use std::num::NonZeroUsize;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use tracing::info;

/// Write to stdout the json for a bunch of Azure DevOps info
pub async fn dump_azure_devops(
    org_url: AzureDevOpsOrganizationUrl,
    auth_selection: AzureDevOpsAuthSelection,
    executor: &CloudTerrastodonExecutor,
) -> eyre::Result<CliOutput> {
    let projects = AzureDevOpsProjectsListRequest {
        org_url: Cow::Borrowed(&org_url),
        auth_selection,
    }
    .execute(executor)
    .await?;

    let users = AzureDevOpsUserLicenseEntitlementListRequest {
        org_url: Cow::Borrowed(&org_url),
        auth_selection,
    }
    .execute(executor)
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
                let result = AzureDevOpsGroupsListRequest {
                    org_url: Cow::Borrowed(&org_url),
                    project: (project.id).into(),
                    auth_selection,
                }
                .execute(&executor)
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
                let result = AzureDevOpsGroupMembersListRequest {
                    org_url: Cow::Borrowed(&org_url),
                    auth_selection,
                    group_id: Cow::Borrowed(&group.descriptor),
                }
                .execute(&executor)
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

    Ok(CliOutput::facet(payload))
}
