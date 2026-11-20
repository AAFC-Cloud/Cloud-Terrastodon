use crate::app::MyApp;
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
            Self::StorageAccount(id) | Self::StorageAccountOutput { id, .. } => {
                Some(id.resource_group_id.subscription_id)
            }
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

    /// Build the Azure scope path from structural IDs and the current account's
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
    // Resource Graph reports these from the immediate parent toward the root.
    let mut hierarchy: Vec<_> = ancestors
        .iter()
        .rev()
        .map(|ancestor| PortalTabTarget::ManagementGroup(ancestor.id()))
        .collect();
    if !matches!(target, PortalTabTarget::Subscription(_))
        && let Some(id) = target.containing_subscription()
    {
        hierarchy.push(PortalTabTarget::Subscription(id));
    }
    if let PortalTabTarget::StorageAccount(id) | PortalTabTarget::StorageAccountOutput { id, .. } =
        target
    {
        hierarchy.push(PortalTabTarget::ResourceGroup(id.resource_group_id.clone()));
    }
    if let PortalTabTarget::StorageAccountOutput { id, .. } = target {
        hierarchy.push(PortalTabTarget::StorageAccount(id.clone()));
    }
    if let PortalTabTarget::ScopeView { scope, .. } = target {
        let overview = scope.overview();
        if hierarchy.last() != Some(&overview) {
            hierarchy.push(overview);
        }
    }
    hierarchy.push(target.clone());
    hierarchy
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::portal_resource_scope::PortalResourceScope;
    use crate::portal_scope_view::PortalScopeView;
    use cloud_terrastodon_app::OutputFormat;
    use cloud_terrastodon_azure::AzureResourceGroupId;
    use cloud_terrastodon_azure::AzureStorageAccountId;

    #[test]
    fn scoped_view_keeps_its_overview_as_the_nearest_breadcrumb() -> eyre::Result<()> {
        let group = AzureResourceGroupId::try_new(
            "00000000-0000-0000-0000-000000000001",
            "synthetic-group",
        )?;
        let view = PortalTabTarget::ScopeView {
            scope: PortalResourceScope::ResourceGroup(group.clone()),
            view: PortalScopeView::Json,
        };
        assert_eq!(
            hierarchy_from_ancestors(&view, &[]),
            [
                PortalTabTarget::Subscription(group.subscription_id),
                PortalTabTarget::ResourceGroup(group),
                view,
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
        let output = PortalTabTarget::StorageAccountOutput {
            id: storage_id.clone(),
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
            PortalTabTarget::Subscription(group_id.subscription_id),
            PortalTabTarget::ResourceGroup(group_id),
            PortalTabTarget::StorageAccount(storage_id),
            output.clone(),
        ];
        assert_eq!(hierarchy_from_ancestors(&output, &[]), without_ancestry);
        assert_eq!(
            hierarchy_from_ancestors(&output, &nearest_parent_first),
            [
                PortalTabTarget::ManagementGroup(ManagementGroupId::from_name("root")),
                PortalTabTarget::ManagementGroup(ManagementGroupId::from_name("workloads")),
            ]
            .into_iter()
            .chain(without_ancestry)
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
                PortalTabTarget::ManagementGroup(ManagementGroupId::from_name("root")),
                target,
            ]
        );
        Ok(())
    }
}
