use clap::Args;
use cloud_terrastodon_azure_devops::AzureDevOpsDescriptor;
use cloud_terrastodon_azure_devops::AzureDevOpsGroup;
use cloud_terrastodon_azure_devops::AzureDevOpsGroupMember;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_all_azure_devops_projects;
use cloud_terrastodon_azure_devops::fetch_azure_devops_group_members;
use cloud_terrastodon_azure_devops::fetch_azure_devops_groups_for_project;
use cloud_terrastodon_azure_devops::get_default_organization_url;
use cloud_terrastodon_command::ParallelFallibleWorkQueue;
use eyre::Result;
use eyre::WrapErr;
use eyre::bail;
use cloud_terrastodon_command::to_writer_pretty;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::io::Write;
use std::io::stdout;
use tracing::info;

/// List users that are transitively members of an Azure DevOps project.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsProjectMemberListArgs {
    /// Project id or project name.
    #[arg(long)]
    pub project: AzureDevOpsProjectArgument<'static>,
}

#[derive(Debug, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
struct AzureDevOpsProjectMember {
    descriptor: AzureDevOpsDescriptor,
    display_name: String,
    principal_name: String,
    mail_address: Option<String>,
    origin: String,
    origin_id: String,
    subject_kind: String,
    permission_objects: Vec<AzureDevOpsProjectPermissionObject>,
}

#[derive(Debug, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
struct AzureDevOpsProjectPermissionObject {
    descriptor: AzureDevOpsDescriptor,
    display_name: String,
    principal_name: String,
    origin: String,
    origin_id: String,
    subject_kind: String,
}

struct AzureDevOpsProjectMemberAccumulator {
    member: AzureDevOpsGroupMember,
    permission_objects: HashMap<AzureDevOpsDescriptor, AzureDevOpsProjectPermissionObject>,
}

impl AzureDevOpsProjectMemberListArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;
        let projects = fetch_all_azure_devops_projects(&org_url).await?;
        let Some(project) = projects.into_iter().find(|p| self.project.matches(p)) else {
            bail!("No project found matching '{}'.", self.project);
        };

        info!(project = %project.name, "Fetching project permission objects");
        let permission_objects = fetch_azure_devops_groups_for_project(&org_url, &project).await?;

        let mut work = ParallelFallibleWorkQueue::new("fetching transitive project members", 4);
        for permission_object in permission_objects {
            let org_url = org_url.clone();
            work.enqueue(async move {
                let members =
                    collect_transitive_user_members(&org_url, &permission_object.descriptor)
                        .await
                        .wrap_err_with(|| {
                            format!(
                                "Failed to collect members for permission object {} ({})",
                                permission_object.display_name, permission_object.descriptor
                            )
                        })?;
                Ok((permission_object, members))
            });
        }

        let mut members_by_descriptor =
            HashMap::<AzureDevOpsDescriptor, AzureDevOpsProjectMemberAccumulator>::new();
        for (permission_object, members) in work.join().await? {
            let permission_object = AzureDevOpsProjectPermissionObject::from(permission_object);
            for member in members {
                members_by_descriptor
                    .entry(member.descriptor.clone())
                    .and_modify(|entry| {
                        entry
                            .permission_objects
                            .entry(permission_object.descriptor.clone())
                            .or_insert_with(|| permission_object.clone());
                    })
                    .or_insert_with(|| {
                        let mut permission_objects = HashMap::new();
                        permission_objects.insert(
                            permission_object.descriptor.clone(),
                            permission_object.clone(),
                        );
                        AzureDevOpsProjectMemberAccumulator {
                            member,
                            permission_objects,
                        }
                    });
            }
        }

        let mut members = members_by_descriptor
            .into_values()
            .map(AzureDevOpsProjectMember::from)
            .collect::<Vec<_>>();
        members.sort_by(|a, b| {
            a.principal_name
                .to_lowercase()
                .cmp(&b.principal_name.to_lowercase())
                .then_with(|| {
                    a.display_name
                        .to_lowercase()
                        .cmp(&b.display_name.to_lowercase())
                })
        });

        let mut out = stdout().lock();
        to_writer_pretty(&mut out, &members)?;
        out.write_all(b"\n")?;

        Ok(())
    }
}

async fn collect_transitive_user_members(
    org_url: &AzureDevOpsOrganizationUrl,
    root_descriptor: &AzureDevOpsDescriptor,
) -> Result<Vec<AzureDevOpsGroupMember>> {
    let mut visited_permission_objects = HashSet::new();
    let mut pending_permission_objects = VecDeque::from([root_descriptor.clone()]);
    let mut users = HashMap::<AzureDevOpsDescriptor, AzureDevOpsGroupMember>::new();

    while let Some(descriptor) = pending_permission_objects.pop_front() {
        if !visited_permission_objects.insert(descriptor.clone()) {
            continue;
        }

        let members = fetch_azure_devops_group_members(org_url, &descriptor)
            .await
            .wrap_err_with(|| {
                format!("Failed to list members for permission object {descriptor}")
            })?;

        for member in members.into_values() {
            if is_user_member(&member) {
                users.entry(member.descriptor.clone()).or_insert(member);
            } else if is_permission_object_member(&member) {
                pending_permission_objects.push_back(member.descriptor);
            }
        }
    }

    Ok(users.into_values().collect())
}

fn is_user_member(member: &AzureDevOpsGroupMember) -> bool {
    member.subject_kind.eq_ignore_ascii_case("user")
        || matches!(member.descriptor, AzureDevOpsDescriptor::EntraUser(_))
}

fn is_permission_object_member(member: &AzureDevOpsGroupMember) -> bool {
    member.subject_kind.eq_ignore_ascii_case("group")
        || matches!(
            member.descriptor,
            AzureDevOpsDescriptor::AzureDevOpsGroup(_) | AzureDevOpsDescriptor::EntraGroup(_)
        )
}

impl From<AzureDevOpsGroup> for AzureDevOpsProjectPermissionObject {
    fn from(group: AzureDevOpsGroup) -> Self {
        AzureDevOpsProjectPermissionObject {
            descriptor: group.descriptor,
            display_name: group.display_name,
            principal_name: group.principal_name,
            origin: group.origin,
            origin_id: group.origin_id,
            subject_kind: group.subject_kind,
        }
    }
}

impl From<AzureDevOpsProjectMemberAccumulator> for AzureDevOpsProjectMember {
    fn from(value: AzureDevOpsProjectMemberAccumulator) -> Self {
        let mut permission_objects = value
            .permission_objects
            .into_values()
            .collect::<Vec<AzureDevOpsProjectPermissionObject>>();
        permission_objects.sort_by(|a, b| {
            a.display_name
                .to_lowercase()
                .cmp(&b.display_name.to_lowercase())
                .then_with(|| a.descriptor.to_string().cmp(&b.descriptor.to_string()))
        });

        AzureDevOpsProjectMember {
            descriptor: value.member.descriptor,
            display_name: value.member.display_name,
            principal_name: value.member.principal_name,
            mail_address: value.member.mail_address,
            origin: value.member.origin,
            origin_id: value.member.origin_id,
            subject_kind: value.member.subject_kind,
            permission_objects,
        }
    }
}
