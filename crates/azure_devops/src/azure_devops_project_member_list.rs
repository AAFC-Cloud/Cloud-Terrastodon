use crate::fetch_all_azure_devops_projects;
use crate::fetch_azure_devops_group_members;
use crate::fetch_azure_devops_groups_for_project;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_devops_types::AzureDevOpsDescriptor;
use cloud_terrastodon_azure_devops_types::AzureDevOpsGroup;
use cloud_terrastodon_azure_devops_types::AzureDevOpsGroupMember;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_command::CacheInvalidatable;
use cloud_terrastodon_command::CacheInvalidatableIntoFuture;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::ParallelFallibleWorkQueue;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use eyre::WrapErr;
use eyre::bail;
use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::future::Future;
use std::future::IntoFuture;
use std::path::PathBuf;
use std::pin::Pin;
use tracing::info;

/// Lists users that are transitively members of an Azure DevOps project.
#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsProjectMemberListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project: AzureDevOpsProjectArgument<'a>,
}

pub fn fetch_azure_devops_project_members<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    project: impl Into<AzureDevOpsProjectArgument<'a>>,
) -> AzureDevOpsProjectMemberListRequest<'a> {
    AzureDevOpsProjectMemberListRequest {
        org_url: Cow::Borrowed(org_url),
        project: project.into(),
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsProjectMemberListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
        })
    }
}

#[derive(Debug, Clone, Arbitrary, facet::Facet)]
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

#[derive(Debug, Clone, Arbitrary, facet::Facet)]
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
    member: AzureDevOpsGroupMember,
    permission_objects: HashMap<AzureDevOpsDescriptor, AzureDevOpsProjectPermissionObject>,
}

#[async_trait]
impl<'a> CacheInvalidatable for AzureDevOpsProjectMemberListRequest<'a> {
    async fn invalidate(&self) -> Result<()> {
        let projects = fetch_all_azure_devops_projects(self.org_url.as_ref()).cache_key();
        let groups =
            fetch_azure_devops_groups_for_project(self.org_url.as_ref(), self.project.clone())
                .cache_key();
        let memberships = CacheKey::new(PathBuf::from_iter([
            "az",
            "devops",
            self.org_url.organization_name.as_ref(),
            "security",
            "group",
            "membership",
            "list",
        ]));

        tokio::try_join!(
            projects.invalidate(),
            groups.invalidate(),
            memberships.invalidate()
        )?;
        Ok(())
    }
}

impl<'a> CacheInvalidatableIntoFuture for AzureDevOpsProjectMemberListRequest<'a> {
    type WithInvalidation = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn with_invalidation(self, invalidate_cache: bool) -> Self::WithInvalidation {
        Box::pin(async move {
            if invalidate_cache {
                self.invalidate().await?;
            }
            self.into_future().await
        })
    }
}

impl<'a> IntoFuture for AzureDevOpsProjectMemberListRequest<'a> {
    type Output = Result<Vec<AzureDevOpsProjectMember>>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let projects = fetch_all_azure_devops_projects(self.org_url.as_ref()).await?;
            let Some(project) = projects
                .into_iter()
                .find(|project| self.project.matches(project))
            else {
                bail!("No project found matching '{}'.", self.project);
            };

            info!(project = %project.name, "Fetching project permission objects");
            let permission_objects =
                fetch_azure_devops_groups_for_project(self.org_url.as_ref(), &project).await?;

            let mut work = ParallelFallibleWorkQueue::new("fetching transitive project members", 4);
            for permission_object in permission_objects {
                let org_url = self.org_url.clone().into_owned();
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

            Ok(members)
        })
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

cloud_terrastodon_registry::register_thing!(AzureDevOpsProjectMemberListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsProjectMemberListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(Vec<AzureDevOpsProjectMember>);
cloud_terrastodon_registry::register_into_future!(
    AzureDevOpsProjectMemberListRequest<'static> => Vec<AzureDevOpsProjectMember>,
    effects = [Read]
);

#[cfg(test)]
mod tests {
    use cloud_terrastodon_registry::{ArbitraryBytes, ProductionKind, functions_from};
    use facet::Facet;

    use super::AzureDevOpsProjectMember;

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
