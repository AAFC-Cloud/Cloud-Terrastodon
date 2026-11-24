use crate::azure_tag_inventory_entry::AzureTagInventoryEntry;
use crate::resource_inventory_row::ResourceInventoryRow;
use cloud_terrastodon_azure::Resource;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::ScopeImpl;
use cloud_terrastodon_azure::SubscriptionId;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Weak;

/// Reverse lookup from a tag to rows in the matching Resource Graph inventory.
/// Azure tag names are case-insensitive and values are case-sensitive.
/// [Azure tag usage](https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/tag-resources#tag-usage-and-recommendations)
pub(crate) struct AzureTagInventory {
    // Resource indexes belong to this exact allocation. A weak reference lets
    // the resource view retire its snapshot when refreshing or switching accounts.
    source: Weak<Vec<Arc<Resource>>>,
    pub entries: BTreeMap<(String, String), AzureTagInventoryEntry>,
    pub subscriptions: BTreeMap<SubscriptionId, String>,
}

impl AzureTagInventory {
    pub fn new(resources: &Arc<Vec<Arc<Resource>>>) -> Self {
        let mut entries = BTreeMap::new();
        let mut subscriptions = BTreeMap::new();
        for (index, resource) in resources.iter().enumerate() {
            if let ScopeImpl::Subscription(id) = &resource.id {
                subscriptions.insert(
                    *id,
                    resource.display_name().unwrap_or(&resource.name).to_owned(),
                );
            }
            if resource.tags.is_empty() {
                continue;
            }
            let subscription = match &resource.id {
                ScopeImpl::Subscription(id) => Some(*id),
                ScopeImpl::ResourceGroup(id) => Some(id.subscription_id),
                _ => ResourceInventoryRow::ancestors(&resource.id.expanded_form()).0,
            };
            if let Some(subscription) = subscription {
                subscriptions
                    .entry(subscription)
                    .or_insert_with(|| subscription.to_string());
            }
            for (name, value) in &resource.tags {
                entries
                    .entry((name.to_lowercase(), value.clone()))
                    .or_insert_with(|| AzureTagInventoryEntry::new(name))
                    .add_resource(name, index, subscription);
            }
        }
        Self {
            source: Arc::downgrade(resources),
            entries,
            subscriptions,
        }
    }

    pub fn matches_snapshot(&self, resources: &Arc<Vec<Arc<Resource>>>) -> bool {
        std::ptr::eq(self.source.as_ptr(), Arc::as_ptr(resources))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_azure::AzureResourceGroupId;
    use cloud_terrastodon_azure::ResourceType;

    #[test]
    fn groups_case_insensitive_names_and_keeps_values_and_subscriptions_distinct()
    -> eyre::Result<()> {
        let first: SubscriptionId = "00000000-0000-0000-0000-000000000001".parse()?;
        let second: SubscriptionId = "00000000-0000-0000-0000-000000000002".parse()?;
        let resource = |id, kind, name: &str, tags: &[(&str, &str)]| {
            Arc::new(Resource {
                id,
                kind,
                name: name.to_owned(),
                location: None,
                tags: tags
                    .iter()
                    .map(|&(name, value)| (name.to_owned(), value.to_owned()))
                    .collect(),
                properties: Default::default(),
            })
        };
        let subscription = ResourceType::MICROSOFT_DOT_RESOURCES_SLASH_SUBSCRIPTIONS;
        let resources = Arc::new(vec![
            resource(
                ScopeImpl::Subscription(first),
                subscription.clone(),
                "First",
                &[("Stage", "Prod"), ("STAGE", "Prod")],
            ),
            resource(
                ScopeImpl::Subscription(second),
                subscription,
                "Second",
                &[("stage", "Prod"), ("owner", "")],
            ),
            resource(
                ScopeImpl::ResourceGroup(AzureResourceGroupId::try_new(second, "sandbox")?),
                ResourceType::MICROSOFT_DOT_RESOURCES_SLASH_SUBSCRIPTIONS_SLASH_RESOURCEGROUPS,
                "sandbox",
                &[("stage", "prod")],
            ),
        ]);
        let index = AzureTagInventory::new(&resources);
        assert_eq!(index.entries.len(), 3);
        let prod = &index.entries[&("stage".into(), "Prod".into())];
        assert_eq!(prod.resource_count(None), 2);
        assert_eq!(
            prod.resources(Some(second)).collect::<Vec<_>>(),
            [(1, Some(second))]
        );
        assert_eq!(
            index.entries[&("stage".into(), "prod".into())].resource_count(Some(first)),
            0
        );
        assert_eq!(index.subscriptions[&second], "Second");
        assert!(index.matches_snapshot(&resources));
        assert!(!index.matches_snapshot(&Arc::new(resources.to_vec())));
        Ok(())
    }
}
