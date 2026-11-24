use crate::app::MyApp;
use crate::portal_azure_content::PortalAzureContent;
use crate::portal_azure_devops_organization_view::PortalAzureDevOpsOrganizationView;
use crate::portal_azure_devops_project_view::PortalAzureDevOpsProjectView;
use crate::portal_page::PortalPage;
use crate::portal_resource_output_target::PortalResourceOutputTarget;
use crate::portal_resource_scope::PortalResourceScope;
use crate::portal_scope_view::PortalScopeView;
use crate::portal_tab_target::PortalTabTarget;
use crate::resource_inventory_row::ResourceInventoryRow;
use crate::workers::load_subscriptions::load_subscriptions;
use cloud_terrastodon_azure::ManagementGroup;
use cloud_terrastodon_azure::ManagementGroupAncestorsChainEntry;
use cloud_terrastodon_azure::ManagementGroupId;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::SubscriptionId;
use cloud_terrastodon_command::app_work::Loadable;
use std::sync::Arc;

impl PortalTabTarget {
    fn containing_subscription(&self) -> Option<SubscriptionId> {
        match self {
            Self::Subscription(id) => Some(*id),
            Self::ResourceGroup(id) => Some(id.subscription_id),
            Self::ScopeView { scope, .. } => scope.overview().containing_subscription(),
            Self::ResourceOutput { resource, .. } => resource.overview().containing_subscription(),
            Self::StorageAccount(id) => Some(id.resource_group_id.subscription_id),
            Self::StorageContainer(id) => {
                Some(id.storage_account_id.resource_group_id.subscription_id)
            }
            Self::Resource(id) => ResourceInventoryRow::ancestors(&id.expanded_form()).0,
            _ => None,
        }
    }
}

impl MyApp {
    pub(crate) fn load_portal_hierarchy_if_needed(&mut self, target: &PortalTabTarget) {
        if target.containing_subscription().is_some()
            && matches!(self.azure.subscriptions, Loadable::NotLoaded)
        {
            load_subscriptions(self);
        }
    }

    /// Build the path from Home through the Azure scope IDs and the current account's
    /// [Resource Graph management group ancestry](https://learn.microsoft.com/en-us/azure/governance/resource-graph/samples/samples-by-category#management-groups).
    pub(crate) fn portal_hierarchy(&self, target: &PortalTabTarget) -> Vec<PortalTabTarget> {
        let ancestors = match target {
            PortalTabTarget::ManagementGroup(id)
            | PortalTabTarget::ScopeView {
                scope: PortalResourceScope::ManagementGroup(id),
                ..
            }
            | PortalTabTarget::ResourceOutput {
                resource:
                    PortalResourceOutputTarget::Scope(PortalResourceScope::ManagementGroup(id)),
                ..
            } => self
                .azure
                .management_group(id)
                .map(|group| group.management_group_ancestors_chain.as_slice())
                .or_else(|| {
                    find_ancestor(self.azure.management_group_chains(), id).map(|(_, tail)| tail)
                }),
            _ => target.containing_subscription().and_then(|id| {
                self.azure
                    .subscriptions
                    .as_loaded()?
                    .iter()
                    .find(|subscription| subscription.id == id)
                    .map(|subscription| subscription.management_group_ancestors_chain.as_slice())
            }),
        };
        hierarchy_from_ancestors(target, ancestors.unwrap_or_default())
    }
}

impl PortalAzureContent {
    pub(crate) fn management_group(&self, id: &ManagementGroupId) -> Option<&ManagementGroup> {
        self.management_groups
            .as_loaded()?
            .iter()
            .find(|group| group.id.name().eq_ignore_ascii_case(id.name()))
            .map(Arc::as_ref)
    }

    pub(crate) fn management_group_title(&self, id: &ManagementGroupId) -> Option<&str> {
        self.management_group(id)
            .map(|group| group.display_name.as_str())
            .or_else(|| {
                find_ancestor(self.management_group_chains(), id)
                    .map(|(group, _)| group.display_name.as_str())
            })
    }

    fn management_group_chains(
        &self,
    ) -> impl Iterator<Item = &[ManagementGroupAncestorsChainEntry]> {
        self.management_groups
            .as_loaded()
            .map(Vec::as_slice)
            .unwrap_or_default()
            .iter()
            .map(|group| group.management_group_ancestors_chain.as_slice())
            .chain(
                self.subscriptions
                    .as_loaded()
                    .map(|subscriptions| subscriptions.as_slice())
                    .unwrap_or_default()
                    .iter()
                    .map(|subscription| subscription.management_group_ancestors_chain.as_slice()),
            )
    }
}

fn find_ancestor<'a>(
    mut chains: impl Iterator<Item = &'a [ManagementGroupAncestorsChainEntry]>,
    id: &ManagementGroupId,
) -> Option<(
    &'a ManagementGroupAncestorsChainEntry,
    &'a [ManagementGroupAncestorsChainEntry],
)> {
    chains.find_map(|chain| {
        let index = chain
            .iter()
            .position(|ancestor| ancestor.name.eq_ignore_ascii_case(id.name()))?;
        Some((&chain[index], &chain[index + 1..]))
    })
}

fn hierarchy_from_ancestors(
    target: &PortalTabTarget,
    ancestors: &[ManagementGroupAncestorsChainEntry],
) -> Vec<PortalTabTarget> {
    if matches!(
        target.page(),
        PortalPage::Accounts | PortalPage::AzureTenants
    ) {
        let mut hierarchy = vec![
            PortalTabTarget::Page(PortalPage::Home),
            PortalTabTarget::Page(PortalPage::About),
        ];
        if matches!(target, PortalTabTarget::AzureTenant(_)) {
            hierarchy.push(PortalTabTarget::Page(PortalPage::AzureTenants));
        }
        hierarchy.push(target.clone());
        return hierarchy;
    }
    if target.page() == PortalPage::AzureDevOpsOrganizations {
        use PortalAzureDevOpsOrganizationView::Overview;
        use PortalAzureDevOpsOrganizationView::Projects;

        let mut hierarchy = vec![PortalTabTarget::Page(PortalPage::Home)];
        match target {
            PortalTabTarget::AzureDevOpsOrganization { organization, view } => {
                hierarchy.push(PortalTabTarget::Page(PortalPage::AzureDevOpsOrganizations));
                if *view != Overview {
                    hierarchy.push(Overview.target(organization));
                }
            }
            PortalTabTarget::AzureDevOpsProject {
                organization,
                project,
                view,
            } => {
                hierarchy.push(PortalTabTarget::Page(PortalPage::AzureDevOpsOrganizations));
                hierarchy.push(Overview.target(organization));
                hierarchy.push(Projects.target(organization));
                if *view != PortalAzureDevOpsProjectView::Overview {
                    hierarchy.push(
                        PortalAzureDevOpsProjectView::Overview.target(organization, *project),
                    );
                }
            }
            _ => {}
        }
        hierarchy.push(target.clone());
        return hierarchy;
    }
    let page = target.page();
    let section = if page.is_audit_section() {
        Some(PortalPage::Auditing)
    } else if page.is_pim_section() {
        Some(PortalPage::Pim)
    } else {
        None
    };
    if let Some(section) = section {
        let mut hierarchy = vec![PortalTabTarget::Page(PortalPage::Home)];
        if page != section {
            hierarchy.push(PortalTabTarget::Page(section));
        }
        hierarchy.push(target.clone());
        return hierarchy;
    }
    if target.page().is_entra() {
        let mut hierarchy = vec![PortalTabTarget::Page(PortalPage::Home)];
        if target.page() != PortalPage::Entra {
            hierarchy.push(PortalTabTarget::Page(PortalPage::Entra));
        }
        if matches!(
            target,
            PortalTabTarget::EntraUser(_)
                | PortalTabTarget::EntraGroup(_)
                | PortalTabTarget::EntraAppRegistration(_)
        ) {
            hierarchy.push(PortalTabTarget::Page(target.page()));
        }
        hierarchy.push(target.clone());
        return hierarchy;
    }
    let overview = match target {
        PortalTabTarget::ScopeView { scope, .. } => Some(scope.overview()),
        PortalTabTarget::ResourceOutput { resource, .. } => Some(resource.overview()),
        PortalTabTarget::StorageContainer(id) => Some(PortalTabTarget::StorageAccount(
            id.storage_account_id.clone(),
        )),
        _ => None,
    };
    let resource = overview.as_ref().unwrap_or(target);
    let subscription = resource.containing_subscription();
    let mut hierarchy = Vec::new();
    if subscription.is_some()
        || matches!(
            resource,
            PortalTabTarget::ManagementGroup(_) | PortalTabTarget::Resource(_)
        )
    {
        hierarchy.push(PortalTabTarget::Page(PortalPage::Home));
    }
    // Resource Graph reports these from the immediate parent toward the root.
    hierarchy.extend(
        ancestors
            .iter()
            .rev()
            .map(|ancestor| PortalTabTarget::ManagementGroup(ancestor.id())),
    );
    if !matches!(resource, PortalTabTarget::Subscription(_))
        && let Some(id) = subscription
    {
        hierarchy.push(PortalTabTarget::Subscription(id));
    }
    let group = match resource {
        PortalTabTarget::StorageAccount(id) => Some(id.resource_group_id.clone()),
        PortalTabTarget::Resource(id) => ResourceInventoryRow::ancestors(&id.expanded_form()).1,
        _ => None,
    };
    if let Some(group) = group {
        hierarchy.push(PortalTabTarget::ResourceGroup(group));
    }
    hierarchy.extend(overview);
    if let PortalTabTarget::StorageContainer(id) = target {
        hierarchy.push(PortalTabTarget::ScopeView {
            scope: PortalResourceScope::StorageAccount(id.storage_account_id.clone()),
            view: PortalScopeView::StorageExplorer,
        });
    }
    hierarchy.push(target.clone());
    hierarchy
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_app::OutputFormat;
    use cloud_terrastodon_azure::AzureResourceGroupId;
    use cloud_terrastodon_azure::AzureStorageAccountBlobContainerId;
    use cloud_terrastodon_azure::AzureStorageAccountId;
    use cloud_terrastodon_azure::ScopeImpl;

    #[test]
    fn devops_project_breadcrumbs_keep_the_organization_scope() -> eyre::Result<()> {
        use PortalAzureDevOpsOrganizationView::Overview;
        use PortalAzureDevOpsOrganizationView::Projects;

        let org = "https://dev.example.test/tfs/Synthetic%20collection".parse()?;
        let projects = Projects.target(&org);
        let id = "00000000-0000-0000-0000-000000000001".parse()?;
        let project = PortalAzureDevOpsProjectView::Overview.target(&org, id);
        let repositories = PortalAzureDevOpsProjectView::Repositories.target(&org, id);
        let parents = [
            PortalTabTarget::Page(PortalPage::Home),
            PortalTabTarget::Page(PortalPage::AzureDevOpsOrganizations),
            Overview.target(&org),
            projects.clone(),
        ];
        assert_eq!(hierarchy_from_ancestors(&projects, &[]), parents);
        assert_eq!(
            hierarchy_from_ancestors(&project, &[]),
            parents
                .iter()
                .cloned()
                .chain([project.clone()])
                .collect::<Vec<_>>()
        );
        assert_eq!(
            hierarchy_from_ancestors(&repositories, &[]),
            parents
                .into_iter()
                .chain([project, repositories])
                .collect::<Vec<_>>()
        );
        Ok(())
    }

    #[test]
    fn nested_resource_blade_keeps_subscription_group_and_resource_breadcrumbs() -> eyre::Result<()>
    {
        let subscription_id = "00000000-0000-0000-0000-000000000001".parse::<SubscriptionId>()?;
        let group = AzureResourceGroupId::try_new(subscription_id, "synthetic-group")?;
        let id = ScopeImpl::from(format!(
            "{}/providers/Microsoft.Compute/galleries/example/images/child/versions/1.0.0",
            group.expanded_form()
        ));
        let scope = PortalResourceScope::Resource(Box::new(id));
        let overview = scope.overview();
        assert_eq!(overview.title(), "1.0.0");
        let activity = PortalTabTarget::ScopeView {
            scope,
            view: PortalScopeView::ActivityLog,
        };
        assert_eq!(
            hierarchy_from_ancestors(&activity, &[]),
            [
                PortalTabTarget::Page(PortalPage::Home),
                PortalTabTarget::Subscription(subscription_id),
                PortalTabTarget::ResourceGroup(group),
                overview,
                activity,
            ],
        );
        Ok(())
    }

    #[test]
    fn tenant_root_has_a_home_breadcrumb() {
        let root = PortalTabTarget::ManagementGroup(ManagementGroupId::from_name("root"));
        assert_eq!(
            hierarchy_from_ancestors(&root, &[]),
            [PortalTabTarget::Page(PortalPage::Home), root],
        );
    }

    #[test]
    fn pages_without_a_resource_hierarchy_have_no_parent_breadcrumb() {
        for page in [PortalPage::Home, PortalPage::About] {
            let target = PortalTabTarget::Page(page);
            assert_eq!(hierarchy_from_ancestors(&target, &[]), [target]);
        }
    }

    #[test]
    fn tenant_detail_breadcrumbs_lead_back_to_its_list() -> eyre::Result<()> {
        let list = PortalTabTarget::Page(PortalPage::AzureTenants);
        let tenant = PortalTabTarget::AzureTenant("00000000-0000-0000-0000-000000000001".parse()?);
        let parents = [
            PortalTabTarget::Page(PortalPage::Home),
            PortalTabTarget::Page(PortalPage::About),
            list.clone(),
        ];
        assert_eq!(hierarchy_from_ancestors(&list, &[]), parents);
        assert_eq!(
            hierarchy_from_ancestors(&tenant, &[]),
            parents.into_iter().chain([tenant]).collect::<Vec<_>>()
        );
        Ok(())
    }

    #[test]
    fn scoped_view_keeps_its_overview_as_the_nearest_breadcrumb() -> eyre::Result<()> {
        let group = AzureResourceGroupId::try_new(
            "00000000-0000-0000-0000-000000000001",
            "synthetic-group",
        )?;
        let view = PortalTabTarget::ScopeView {
            scope: PortalResourceScope::ResourceGroup(group.clone()),
            view: PortalScopeView::ActivityLog,
        };
        assert_eq!(
            hierarchy_from_ancestors(&view, &[]),
            [
                PortalTabTarget::Page(PortalPage::Home),
                PortalTabTarget::Subscription(group.subscription_id),
                PortalTabTarget::ResourceGroup(group),
                view,
            ],
        );
        Ok(())
    }

    #[test]
    fn scoped_output_has_the_overview_as_its_nearest_breadcrumb() -> eyre::Result<()> {
        let group = AzureResourceGroupId::try_new(
            "00000000-0000-0000-0000-000000000001",
            "synthetic-group",
        )?;
        let subscription = PortalTabTarget::Subscription(group.subscription_id);
        let subscription_output = PortalTabTarget::ResourceOutput {
            resource: PortalResourceOutputTarget::Scope(PortalResourceScope::Subscription(
                group.subscription_id,
            )),
            format: OutputFormat::FacetPretty,
        };
        assert_eq!(
            hierarchy_from_ancestors(&subscription_output, &[]),
            [
                PortalTabTarget::Page(PortalPage::Home),
                subscription.clone(),
                subscription_output,
            ],
        );

        let group_output = PortalTabTarget::ResourceOutput {
            resource: PortalResourceOutputTarget::Scope(PortalResourceScope::ResourceGroup(
                group.clone(),
            )),
            format: OutputFormat::Text,
        };
        assert_eq!(
            hierarchy_from_ancestors(&group_output, &[]),
            [
                PortalTabTarget::Page(PortalPage::Home),
                subscription,
                PortalTabTarget::ResourceGroup(group),
                group_output,
            ],
        );
        Ok(())
    }

    #[test]
    fn storage_output_hierarchy_uses_structural_ids_and_root_first_ancestry() -> eyre::Result<()> {
        let group_id = AzureResourceGroupId::try_new(
            "00000000-0000-0000-0000-000000000001",
            "synthetic-group",
        )?;
        let storage_id = AzureStorageAccountId::try_new(group_id.clone(), "syntheticstorage")?;
        let output = PortalTabTarget::ResourceOutput {
            resource: PortalResourceOutputTarget::StorageAccount(storage_id.clone()),
            format: OutputFormat::Json,
        };
        let nearest_parent_first = [
            ManagementGroupAncestorsChainEntry {
                name: "workloads".to_owned(),
                display_name: "Workloads".to_owned(),
            },
            ManagementGroupAncestorsChainEntry {
                name: "root".to_owned(),
                display_name: "Root".to_owned(),
            },
        ];
        let without_ancestry = vec![
            PortalTabTarget::Page(PortalPage::Home),
            PortalTabTarget::Subscription(group_id.subscription_id),
            PortalTabTarget::ResourceGroup(group_id),
            PortalTabTarget::StorageAccount(storage_id),
            output.clone(),
        ];
        assert_eq!(hierarchy_from_ancestors(&output, &[]), without_ancestry);
        assert_eq!(
            hierarchy_from_ancestors(&output, &nearest_parent_first),
            [
                PortalTabTarget::Page(PortalPage::Home),
                PortalTabTarget::ManagementGroup(ManagementGroupId::from_name("root")),
                PortalTabTarget::ManagementGroup(ManagementGroupId::from_name("workloads")),
            ]
            .into_iter()
            .chain(without_ancestry.into_iter().skip(1))
            .collect::<Vec<_>>()
        );

        let management_group = ManagementGroupId::from_name("workloads");
        let target = PortalTabTarget::ManagementGroup(management_group.clone());
        let (group, ancestors) = find_ancestor(
            std::iter::once(&nearest_parent_first[..]),
            &management_group,
        )
        .ok_or_else(|| eyre::eyre!("workloads was not in the known subscription chain"))?;
        assert_eq!(group.display_name, "Workloads");
        assert_eq!(
            hierarchy_from_ancestors(&target, ancestors),
            [
                PortalTabTarget::Page(PortalPage::Home),
                PortalTabTarget::ManagementGroup(ManagementGroupId::from_name("root")),
                target,
            ]
        );
        Ok(())
    }

    #[test]
    fn storage_container_breadcrumbs_include_storage_explorer_and_its_account() -> eyre::Result<()>
    {
        let group = AzureResourceGroupId::try_new(
            "00000000-0000-0000-0000-000000000001",
            "synthetic-group",
        )?;
        let account = AzureStorageAccountId::try_new(group.clone(), "syntheticstore")?;
        let container = PortalTabTarget::StorageContainer(AzureStorageAccountBlobContainerId {
            storage_account_id: account.clone(),
            container_name: "synthetic-container".parse()?,
        });
        assert_eq!(
            container.containing_subscription(),
            Some(group.subscription_id)
        );
        assert_eq!(
            hierarchy_from_ancestors(&container, &[]),
            [
                PortalTabTarget::Page(PortalPage::Home),
                PortalTabTarget::Subscription(group.subscription_id),
                PortalTabTarget::ResourceGroup(group),
                PortalTabTarget::StorageAccount(account.clone()),
                PortalTabTarget::ScopeView {
                    scope: PortalResourceScope::StorageAccount(account),
                    view: PortalScopeView::StorageExplorer,
                },
                container,
            ]
        );
        Ok(())
    }
}
