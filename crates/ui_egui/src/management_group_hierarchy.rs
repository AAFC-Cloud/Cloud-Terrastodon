use cloud_terrastodon_azure::ManagementGroup;
use cloud_terrastodon_azure::ManagementGroupAncestorsChainEntry;
use cloud_terrastodon_azure::ManagementGroupId;
use cloud_terrastodon_azure::Subscription;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

/// The management groups and subscriptions visible in the selected tenant's inventory.
pub(crate) struct ManagementGroupHierarchy<'a> {
    groups: HashMap<String, ManagementGroupAncestorsChainEntry>,
    parents: HashMap<String, String>,
    children: HashMap<String, Vec<String>>,
    subscriptions: HashMap<String, Vec<&'a Subscription>>,
    subscription_counts: HashMap<String, usize>,
}

impl<'a> ManagementGroupHierarchy<'a> {
    pub(crate) fn new(
        groups: &[Arc<ManagementGroup>],
        subscriptions: &'a [Arc<Subscription>],
    ) -> Self {
        let mut hierarchy = Self {
            groups: HashMap::new(),
            parents: HashMap::new(),
            children: HashMap::new(),
            subscriptions: HashMap::new(),
            subscription_counts: HashMap::new(),
        };
        // Explicit group records take precedence over relationships inferred from other chains.
        for group in groups {
            let key = group.name().to_lowercase();
            hierarchy.groups.insert(
                key.clone(),
                ManagementGroupAncestorsChainEntry {
                    name: group.name().to_owned(),
                    display_name: group.display_name.clone(),
                },
            );
            if let Some(parent) = group.management_group_ancestors_chain.first() {
                hierarchy.link(key, parent.name.to_lowercase());
            }
        }
        for group in groups {
            hierarchy.add_ancestors(&group.management_group_ancestors_chain);
        }

        let mut seen_subscriptions = HashSet::new();
        for subscription in subscriptions {
            let ancestors = &subscription.management_group_ancestors_chain;
            hierarchy.add_ancestors(ancestors);
            if let Some(parent) = ancestors.first()
                && seen_subscriptions.insert(subscription.id)
            {
                hierarchy
                    .subscriptions
                    .entry(parent.name.to_lowercase())
                    .or_default()
                    .push(subscription);
            }
        }

        for (child, parent) in &hierarchy.parents {
            hierarchy
                .children
                .entry(parent.clone())
                .or_default()
                .push(child.clone());
        }
        for siblings in hierarchy.children.values_mut() {
            siblings.sort_by_cached_key(|key| {
                let name = hierarchy
                    .groups
                    .get(key)
                    .map(|group| group.display_name.to_lowercase());
                (name, key.clone())
            });
        }
        for (parent, subscriptions) in &mut hierarchy.subscriptions {
            subscriptions.sort_by_cached_key(|subscription| {
                (subscription.name.as_str().to_lowercase(), subscription.id)
            });
            let mut group = Some(parent.as_str());
            while let Some(key) = group {
                *hierarchy
                    .subscription_counts
                    .entry(key.to_owned())
                    .or_default() += subscriptions.len();
                group = hierarchy.parents.get(key).map(String::as_str);
            }
        }
        hierarchy
    }

    pub(crate) fn group(
        &self,
        id: &ManagementGroupId,
    ) -> Option<&ManagementGroupAncestorsChainEntry> {
        self.groups.get(&id.name().to_lowercase())
    }

    pub(crate) fn child_groups<'tree>(
        &'tree self,
        id: &ManagementGroupId,
    ) -> impl Iterator<Item = &'tree ManagementGroupAncestorsChainEntry> + use<'tree, 'a> {
        self.children
            .get(&id.name().to_lowercase())
            .into_iter()
            .flatten()
            .filter_map(|key| self.groups.get(key))
    }

    pub(crate) fn subscriptions(&self, id: &ManagementGroupId) -> &[&'a Subscription] {
        self.subscriptions
            .get(&id.name().to_lowercase())
            .map_or(&[], Vec::as_slice)
    }

    pub(crate) fn subscription_count(&self, id: &ManagementGroupId) -> usize {
        self.subscription_counts
            .get(&id.name().to_lowercase())
            .copied()
            .unwrap_or_default()
    }

    pub(crate) fn ancestors(
        &self,
        id: &ManagementGroupId,
    ) -> Vec<&ManagementGroupAncestorsChainEntry> {
        let mut ancestors = Vec::new();
        let mut parent = self.parents.get(&id.name().to_lowercase());
        while let Some(key) = parent {
            if let Some(group) = self.groups.get(key) {
                ancestors.push(group);
            }
            parent = self.parents.get(key);
        }
        ancestors.reverse();
        ancestors
    }

    fn add_ancestors(&mut self, ancestors: &[ManagementGroupAncestorsChainEntry]) {
        // Chains run from the direct parent toward the root. They can describe
        // groups the selected account cannot load as standalone records.
        let mut child = None;
        for ancestor in ancestors {
            let key = ancestor.name.to_lowercase();
            self.groups
                .entry(key.clone())
                .or_insert_with(|| ancestor.clone());
            if let Some(child) = child.replace(key.clone()) {
                self.link(child, key);
            }
        }
    }

    fn link(&mut self, child: String, parent: String) {
        if self.parents.contains_key(&child) {
            return;
        }
        // Inventories can disagree while a group is moving. Keep a tree even
        // when combining independently loaded hints produces a cycle.
        let mut ancestor = Some(parent.as_str());
        while let Some(key) = ancestor {
            if key == child {
                return;
            }
            ancestor = self.parents.get(key).map(String::as_str);
        }
        self.parents.insert(child, parent);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_azure::ManagementGroupAncestorsChain;
    use facet_reflect::Partial;

    fn chain(names: &[&str]) -> eyre::Result<ManagementGroupAncestorsChain> {
        let entries = names
            .iter()
            .map(|name| ManagementGroupAncestorsChainEntry {
                name: (*name).into(),
                display_name: (*name).into(),
            })
            .collect::<Vec<_>>();
        Ok(Partial::alloc::<ManagementGroupAncestorsChain>()?
            .begin_inner()?
            .set(entries)?
            .end()?
            .build()?
            .materialize()?)
    }

    #[test]
    fn hierarchy_combines_direct_subscriptions_and_sparse_ancestors() -> eyre::Result<()> {
        let tenant_id = "00000000-0000-0000-0000-000000000001".parse()?;
        let groups = [
            Arc::new(ManagementGroup {
                id: ManagementGroupId::from_name("prod"),
                display_name: "Production".into(),
                tenant_id,
                management_group_ancestors_chain: chain(&["root"])?,
            }),
            Arc::new(ManagementGroup {
                id: ManagementGroupId::from_name("dev"),
                display_name: "Development".into(),
                tenant_id,
                management_group_ancestors_chain: chain(&["Root"])?,
            }),
        ];
        let subscriptions = [
            Arc::new(Subscription {
                id: "00000000-0000-0000-0000-000000000002".parse()?,
                name: "Nested subscription".parse()?,
                tenant_id,
                management_group_ancestors_chain: chain(&["Shared", "PROD"])?,
                tags: Default::default(),
            }),
            Arc::new(Subscription {
                id: "00000000-0000-0000-0000-000000000003".parse()?,
                name: "Direct subscription".parse()?,
                tenant_id,
                management_group_ancestors_chain: chain(&["Root"])?,
                tags: Default::default(),
            }),
        ];
        let hierarchy = ManagementGroupHierarchy::new(&groups, &subscriptions);
        let root = ManagementGroupId::from_name("ROOT");
        let prod = ManagementGroupId::from_name("prod");
        let shared = ManagementGroupId::from_name("shared");
        assert_eq!(
            hierarchy
                .child_groups(&root)
                .map(|group| group.name.as_str())
                .collect::<Vec<_>>(),
            ["dev", "prod"]
        );
        assert_eq!(
            hierarchy
                .child_groups(&prod)
                .next()
                .map(|group| group.name.as_str()),
            Some("Shared")
        );
        assert_eq!(hierarchy.subscriptions(&root), &[subscriptions[1].as_ref()]);
        assert!(hierarchy.subscriptions(&prod).is_empty());
        assert_eq!(
            hierarchy.subscriptions(&shared),
            &[subscriptions[0].as_ref()]
        );
        assert_eq!(hierarchy.subscription_count(&root), 2);
        assert_eq!(hierarchy.subscription_count(&prod), 1);
        assert_eq!(
            hierarchy
                .ancestors(&shared)
                .iter()
                .map(|group| group.name.as_str())
                .collect::<Vec<_>>(),
            ["root", "prod"]
        );
        Ok(())
    }

    #[test]
    fn conflicting_chains_cannot_introduce_a_cycle() -> eyre::Result<()> {
        let tenant_id = "00000000-0000-0000-0000-000000000001".parse()?;
        let groups = [
            Arc::new(ManagementGroup {
                id: ManagementGroupId::from_name("child"),
                display_name: "Child".into(),
                tenant_id,
                management_group_ancestors_chain: chain(&["parent"])?,
            }),
            Arc::new(ManagementGroup {
                id: ManagementGroupId::from_name("parent"),
                display_name: "Parent".into(),
                tenant_id,
                management_group_ancestors_chain: chain(&["child"])?,
            }),
        ];
        let hierarchy = ManagementGroupHierarchy::new(&groups, &[]);
        assert!(hierarchy.ancestors(&groups[1].id).is_empty());
        assert_eq!(
            hierarchy
                .ancestors(&groups[0].id)
                .iter()
                .map(|group| group.name.as_str())
                .collect::<Vec<_>>(),
            ["parent"]
        );
        Ok(())
    }
}
