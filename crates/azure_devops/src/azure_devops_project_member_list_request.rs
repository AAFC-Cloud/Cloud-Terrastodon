use crate::AzureDevOpsGroupMembersListRequest;
use crate::AzureDevOpsGroupsListRequest;
use crate::AzureDevOpsProjectsListRequest;
use crate::azure_devops_project_member_accumulator::AzureDevOpsProjectMemberAccumulator;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops_types::AzureDevOpsDescriptor;
use cloud_terrastodon_azure_devops_types::AzureDevOpsGroupMember;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProject;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectMember;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectPermissionObject;
use cloud_terrastodon_execution::CacheInvalidatableRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ConcurrentIteratorExt;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use eyre::OptionExt;
use eyre::Result;
use eyre::WrapErr;
use facet::Facet;
use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use tracing::info;

/// Lists users that are transitively members of an Azure DevOps project.
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsProjectMemberListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project: AzureDevOpsProjectArgument<'a>,
    pub auth_selection: AzureDevOpsAuthSelection,
}

impl<'a> Arbitrary<'a> for AzureDevOpsProjectMemberListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
            auth_selection: AzureDevOpsAuthSelection::arbitrary(u)?,
        })
    }
}

impl AzureDevOpsProjectMemberListRequest<'_> {
    /// Resolve name selectors before choosing the group's cache scope. Refresh
    /// and ordinary execution must use the same project identity.
    fn groups_request(
        &self,
        projects: &[Arc<AzureDevOpsProject>],
    ) -> Result<AzureDevOpsGroupsListRequest<'_>> {
        let project = projects
            .iter()
            .find(|project| self.project.matches(project))
            .ok_or_else(|| eyre::eyre!("No project found matching '{}'.", self.project))?;
        Ok(AzureDevOpsGroupsListRequest {
            org_url: Cow::Borrowed(self.org_url.as_ref()),
            project: project.id.into(),
            auth_selection: self.auth_selection,
        })
    }
}

impl CacheInvalidatableRequest for AzureDevOpsProjectMemberListRequest<'_> {
    fn invalidate<'a>(&'a self, executor: &'a CloudTerrastodonExecutor) -> ExecutionFuture<'a, ()> {
        Box::pin(async move {
            let projects = AzureDevOpsProjectsListRequest {
                org_url: Cow::Borrowed(self.org_url.as_ref()),
                auth_selection: self.auth_selection,
            }
            .execute(executor)
            .with_invalidation(true)
            .await?;
            let groups = self
                .groups_request(&projects)?
                .execute(executor)
                .with_invalidation(true)
                .await?;
            // A group's own contract owns its key. Refresh only the selected
            // project's reachable memberships, including newly added subgroups.
            for group in groups {
                collect_transitive_user_members(
                    self.org_url.as_ref(),
                    &group.descriptor,
                    self.auth_selection,
                    true,
                    executor,
                )
                .await?;
            }
            Ok(())
        })
    }
}

impl<'a> ExecutableRequest for AzureDevOpsProjectMemberListRequest<'a> {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> Result<CloudTerrastodonExecutor, Self::Error> {
        executor.validate_authentication_target(
            &self.auth_selection.into(),
            &self.org_url.api_url("dev.azure.com", "_apis", &[])?,
        )?;
        executor.with_authentication_lease(&self.auth_selection.into())
    }

    type Error = eyre::Report;
    type Output = Vec<AzureDevOpsProjectMember>;

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(executor.cancellable_scope(move |executor| async move {
            let projects = AzureDevOpsProjectsListRequest {
                org_url: Cow::Borrowed(self.org_url.as_ref()),
                auth_selection: self.auth_selection,
            }
            .execute(&executor)
            .await?;
            info!(project = %self.project, "Fetching project permission objects");
            let permission_objects = self.groups_request(&projects)?.execute(&executor).await?;

            let remaining = AtomicUsize::new(permission_objects.len());
            // Retain complete-all behavior for independent permission-object
            // failures, while caller cancellation stops optional membership reads.
            let members = permission_objects
                .into_iter()
                .map_concurrent(
                    &executor,
                    NonZeroUsize::new(4).ok_or_eyre("membership concurrency must be nonzero")?,
                    async |executor, permission_object| {
                        let result = collect_transitive_user_members(
                            self.org_url.as_ref(),
                            &permission_object.descriptor,
                            self.auth_selection,
                            false,
                            &executor,
                        )
                        .await
                        .wrap_err_with(|| {
                            format!(
                                "Failed to collect members for permission object {} ({})",
                                permission_object.display_name, permission_object.descriptor
                            )
                        })
                        .map(|members| (permission_object, members));
                        info!(
                            "Fetching transitive project members, {} permission objects remain",
                            remaining.fetch_sub(1, Ordering::Relaxed) - 1
                        );
                        eyre::Ok(result)
                    },
                )
                .await?
                .into_iter()
                .collect::<Result<Vec<_>>>()?;
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
    auth_selection: AzureDevOpsAuthSelection,
    invalidate: bool,
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

        let members = AzureDevOpsGroupMembersListRequest {
            org_url: Cow::Borrowed(org_url),
            auth_selection,
            group_id: Cow::Borrowed(&descriptor),
        }
        .execute(executor)
        .with_invalidation(invalidate)
        .await
        .wrap_err_with(|| format!("Failed to list members for permission object {descriptor}"))?;

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

cloud_terrastodon_registry::register_thing!(AzureDevOpsProjectMemberListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsProjectMemberListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureDevOpsProjectMemberListRequest<'static> => Vec<AzureDevOpsProjectMember>,
    effects = [Read]
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_executor::authentication_selection;
    use chrono::DateTime;
    use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectId;
    use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectName;
    use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectState;
    use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectVisibility;
    use cloud_terrastodon_cache::HasCacheKey;
    use cloud_terrastodon_registry::ArbitraryBytes;
    use cloud_terrastodon_registry::ProductionKind;
    use cloud_terrastodon_registry_execution::functions_from;
    use facet::Facet;

    fn project(id: &str, name: &str) -> AzureDevOpsProject {
        AzureDevOpsProject {
            abbreviation: None,
            default_team_image_url: None,
            description: None,
            id: id.parse::<AzureDevOpsProjectId>().unwrap(),
            last_update_time: DateTime::UNIX_EPOCH,
            name: AzureDevOpsProjectName::try_new(name).unwrap(),
            revision: 1,
            state: AzureDevOpsProjectState::WellFormed,
            url: format!("https://dev.azure.com/synthetic/_apis/projects/{id}"),
            visibility: AzureDevOpsProjectVisibility::Private,
        }
    }

    #[test]
    fn name_and_id_selection_share_only_the_resolved_project_scope() -> Result<()> {
        let projects = [
            Arc::new(project("00000000-0000-0000-0000-000000000001", "Selected")),
            Arc::new(project("00000000-0000-0000-0000-000000000002", "Unrelated")),
        ];
        let request = AzureDevOpsProjectMemberListRequest {
            org_url: Cow::Owned("synthetic".parse()?),
            project: "selected".parse()?,
            auth_selection: authentication_selection(),
        };
        let by_name = request.groups_request(&projects)?.cache_key()?.scope;
        let by_id = AzureDevOpsProjectMemberListRequest {
            project: projects[0].id.into(),
            ..request.clone()
        }
        .groups_request(&projects)?
        .cache_key()?
        .scope;
        let unrelated = AzureDevOpsProjectMemberListRequest {
            project: projects[1].id.into(),
            ..request.clone()
        }
        .groups_request(&projects)?
        .cache_key()?
        .scope;
        assert_eq!(by_name, by_id);
        assert_ne!(by_name, unrelated);
        assert!(
            AzureDevOpsProjectMemberListRequest {
                project: "Missing".parse()?,
                ..request
            }
            .groups_request(&projects)
            .is_err()
        );
        Ok(())
    }

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

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureDevOpsProjectMemberListRequest<'static>,
    name = "azure-devops-project-member-list",
    cached
);
