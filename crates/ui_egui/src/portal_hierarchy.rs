use crate::app::MyApp;
use crate::portal_page::PortalPage;
use crate::portal_tab_target::PortalTabTarget;
use crate::workers::load_subscriptions::load_subscriptions;
use cloud_terrastodon_azure::ManagementGroup;
use cloud_terrastodon_azure::ManagementGroupAncestorsChainEntry;
use cloud_terrastodon_azure::ManagementGroupId;
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
            _ => None,
        }
    }
}

impl MyApp {
    pub(crate) fn load_portal_hierarchy_if_needed(&mut self, target: &PortalTabTarget) {
        if target.containing_subscription().is_some()
            && matches!(self.subscriptions, Loadable::NotLoaded)
        {
            load_subscriptions(self);
        }
    }

    /// Build the path from Home through the Azure scope IDs and the current account's
    /// [Resource Graph management group ancestry](https://learn.microsoft.com/en-us/azure/governance/resource-graph/samples/samples-by-category#management-groups).
    pub(crate) fn portal_hierarchy(&self, target: &PortalTabTarget) -> Vec<PortalTabTarget> {
        let ancestors = match target {
            PortalTabTarget::ManagementGroup(id) => self
                .management_group(id)
                .map(|group| group.management_group_ancestors_chain.as_slice())
                .or_else(|| {
                    find_ancestor(self.management_group_chains(), id).map(|(_, tail)| tail)
                }),
            _ => target.containing_subscription().and_then(|id| {
                self.subscriptions
                    .as_loaded()?
                    .iter()
                    .find(|subscription| subscription.id == id)
                    .map(|subscription| subscription.management_group_ancestors_chain.as_slice())
            }),
        };
        hierarchy_from_ancestors(target, ancestors.unwrap_or_default())
    }

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
    let overview = match target {
        PortalTabTarget::ScopeView { scope, .. } => Some(scope.overview()),
        PortalTabTarget::ResourceOutput { resource, .. } => Some(resource.overview()),
        _ => None,
    };
    let resource = overview.as_ref().unwrap_or(target);
    let subscription = resource.containing_subscription();
    let mut hierarchy = Vec::new();
    if subscription.is_some() || matches!(resource, PortalTabTarget::ManagementGroup(_)) {
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
    if let PortalTabTarget::StorageAccount(id) = resource {
        hierarchy.push(PortalTabTarget::ResourceGroup(id.resource_group_id.clone()));
    }
    hierarchy.extend(overview);
    hierarchy.push(target.clone());
    hierarchy
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::portal_resource_output_target::PortalResourceOutputTarget;
    use crate::portal_resource_scope::PortalResourceScope;
    use crate::portal_scope_view::PortalScopeView;
    use cloud_terrastodon_app::OutputFormat;
    use cloud_terrastodon_azure::AzureResourceGroupId;
    use cloud_terrastodon_azure::AzureStorageAccountId;

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
        for page in [PortalPage::Home, PortalPage::Accounts] {
            let target = PortalTabTarget::Page(page);
            assert_eq!(hierarchy_from_ancestors(&target, &[]), [target]);
        }
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
}
