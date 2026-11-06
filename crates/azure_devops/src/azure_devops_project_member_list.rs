use crate::fetch_all_azure_devops_projects;
use crate::fetch_azure_devops_group_members;
use crate::fetch_azure_devops_groups_for_project;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth::AzureDevOpsAuthContext;
use cloud_terrastodon_azure_devops_types::AzureDevOpsDescriptor;
use cloud_terrastodon_azure_devops_types::AzureDevOpsGroup;
use cloud_terrastodon_azure_devops_types::AzureDevOpsGroupMember;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_cache::CacheInvalidationKeys;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_command::ParallelFallibleWorkQueue;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use eyre::Result;
use eyre::WrapErr;
use eyre::bail;
use facet::Facet;
use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

/// Lists users that are transitively members of an Azure DevOps project.
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsProjectMemberListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project: AzureDevOpsProjectArgument<'a>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
}

#[track_caller]
pub fn fetch_azure_devops_project_members<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    project: impl Into<AzureDevOpsProjectArgument<'a>>,
    auth_context: &'a AzureDevOpsAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, AzureDevOpsProjectMemberListRequest<'a>> {
    executor.execute(AzureDevOpsProjectMemberListRequest {
        org_url: Cow::Borrowed(org_url),
        project: project.into(),
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl<'a> Arbitrary<'a> for AzureDevOpsProjectMemberListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
        })
    }
}

#[derive(Debug, Clone, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsProjectMember {
    pub descriptor: AzureDevOpsDescriptor,
    pub display_name: String,
    pub principal_name: String,
    pub mail_address: Option<String>,
    pub origin: String,
    pub origin_id: String,
    pub subject_kind: String,
    pub permission_objects: Vec<AzureDevOpsProjectPermissionObject>,
}

#[derive(Debug, Clone, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsProjectPermissionObject {
    pub descriptor: AzureDevOpsDescriptor,
    pub display_name: String,
    pub principal_name: String,
    pub origin: String,
    pub origin_id: String,
    pub subject_kind: String,
}

struct AzureDevOpsProjectMemberAccumulator {
    member: Arc<AzureDevOpsGroupMember>,
    permission_objects: HashMap<AzureDevOpsDescriptor, AzureDevOpsProjectPermissionObject>,
}
impl<'a> CacheInvalidationKeys for AzureDevOpsProjectMemberListRequest<'a> {
    fn cache_keys_to_invalidate(&self) -> Result<Vec<CacheKey>> {
        let projects = crate::AzureDevOpsProjectsListRequest {
            org_url: Cow::Borrowed(self.org_url.as_ref()),
            auth_selection: self.auth_context.selection()?,
        }
        .cache_key()?;
        let groups = crate::AzureDevOpsGroupsListRequest {
            org_url: Cow::Borrowed(self.org_url.as_ref()),
            project: self.project.clone(),
            auth_context: Cow::Borrowed(self.auth_context.as_ref()),
        }
        .cache_key()?;
        let memberships = CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "devops",
            self.org_url.name(),
            "security",
            "group",
            "membership",
            "list",
        ]))?;
        Ok(vec![projects, groups, memberships])
    }
}

impl<'a> ExecutableRequest for AzureDevOpsProjectMemberListRequest<'a> {
    type Error = eyre::Report;
    type Output = Vec<AzureDevOpsProjectMember>;

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(executor.cancellable_scope(move |executor| async move {
            let projects = fetch_all_azure_devops_projects(
                self.org_url.as_ref(),
                self.auth_context.selection()?,
                &executor,
            )
            .await?;
            let Some(project) = projects
                .iter()
                .find(|project| self.project.matches(project))
            else {
                bail!("No project found matching '{}'.", self.project);
            };

            info!(project = %project.name, "Fetching project permission objects");
            let permission_objects = fetch_azure_devops_groups_for_project(
                self.org_url.as_ref(),
                project.as_ref(),
                self.auth_context.as_ref(),
                &executor,
            )
            .await?;

            let mut work = ParallelFallibleWorkQueue::new("fetching transitive project members", 4);
            for permission_object in permission_objects {
                executor.cancellation().bail_if_cancelled()?;
                let org_url = self.org_url.as_ref().clone();
                let auth_context = self.auth_context.as_ref().clone();
                work.enqueue(executor.cancellable_scope(move |executor| async move {
                    let members = collect_transitive_user_members(
                        &org_url,
                        &permission_object.descriptor,
                        &auth_context,
                        &executor,
                    )
                    .await
                    .wrap_err_with(|| {
                        format!(
                            "Failed to collect members for permission object {} ({})",
                            permission_object.display_name, permission_object.descriptor
                        )
                    })?;
                    Ok((permission_object, members))
                }));
            }

            let members = work.join().await?;
            let mut members_by_descriptor =
                HashMap::<AzureDevOpsDescriptor, AzureDevOpsProjectMemberAccumulator>::new();
            for (permission_object, members) in members {
                let permission_object =
                    AzureDevOpsProjectPermissionObject::from(permission_object.as_ref());
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

            Ok(members)
        }))
    }
}

async fn collect_transitive_user_members(
    org_url: &AzureDevOpsOrganizationUrl,
    root_descriptor: &AzureDevOpsDescriptor,
    auth_context: &AzureDevOpsAuthContext,
    executor: &CloudTerrastodonExecutor,
) -> Result<Vec<Arc<AzureDevOpsGroupMember>>> {
    let mut visited_permission_objects = HashSet::new();
    let mut pending_permission_objects = VecDeque::from([root_descriptor.clone()]);
    let mut users = HashMap::<AzureDevOpsDescriptor, Arc<AzureDevOpsGroupMember>>::new();

    while let Some(descriptor) = pending_permission_objects.pop_front() {
        executor.cancellation().bail_if_cancelled()?;
        if !visited_permission_objects.insert(descriptor.clone()) {
            continue;
        }

        let members =
            fetch_azure_devops_group_members(org_url, &descriptor, auth_context, executor)
                .await
                .wrap_err_with(|| {
                    format!("Failed to list members for permission object {descriptor}")
                })?;

        for member in members.values() {
            if is_user_member(member) {
                users
                    .entry(member.descriptor.clone())
                    .or_insert_with(|| Arc::clone(member));
            } else if is_permission_object_member(member) {
                pending_permission_objects.push_back(member.descriptor.clone());
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

impl From<&AzureDevOpsGroup> for AzureDevOpsProjectPermissionObject {
    fn from(group: &AzureDevOpsGroup) -> Self {
        AzureDevOpsProjectPermissionObject {
            descriptor: group.descriptor.clone(),
            display_name: group.display_name.clone(),
            principal_name: group.principal_name.clone(),
            origin: group.origin.clone(),
            origin_id: group.origin_id.clone(),
            subject_kind: group.subject_kind.clone(),
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
            descriptor: value.member.descriptor.clone(),
            display_name: value.member.display_name.clone(),
            principal_name: value.member.principal_name.clone(),
            mail_address: value.member.mail_address.clone(),
            origin: value.member.origin.clone(),
            origin_id: value.member.origin_id.clone(),
            subject_kind: value.member.subject_kind.clone(),
            permission_objects,
        }
    }
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsProjectMemberListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsProjectMemberListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsProjectMember);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureDevOpsProjectMemberListRequest<'static> => Vec<AzureDevOpsProjectMember>,
    effects = [Read]
);

#[cfg(test)]
mod tests {
    use super::AzureDevOpsProjectMember;
    use cloud_terrastodon_registry::ArbitraryBytes;
    use cloud_terrastodon_registry::ProductionKind;
    use cloud_terrastodon_registry_execution::functions_from;
    use facet::Facet;

    #[test]
    fn project_member_list_has_an_arbitrary_response_constructor() {
        assert!(
            functions_from(ArbitraryBytes::SHAPE)
                .into_iter()
                .any(|function| {
                    function.production_kind(Vec::<AzureDevOpsProjectMember>::SHAPE)
                        == Some(ProductionKind::Exact)
                })
        );
    }
}
