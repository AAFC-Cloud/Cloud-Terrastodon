use crate::reflected_table_group::ReflectedTableGroup;
use crate::resource_inventory_row::ResourceInventoryRow;
use eframe::egui::Id;

/// The ways to group resources using the inventory's existing fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ResourceInventoryGrouping {
    Type,
    ResourceGroup,
    Location,
    Subscription,
}

impl ResourceInventoryGrouping {
    pub const ALL: [Self; 4] = [
        Self::Type,
        Self::ResourceGroup,
        Self::Location,
        Self::Subscription,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Type => "Type",
            Self::ResourceGroup => "Resource group",
            Self::Location => "Location",
            Self::Subscription => "Subscription",
        }
    }

    pub fn group(self, row: &ResourceInventoryRow) -> ReflectedTableGroup {
        let (id, label) = match self {
            Self::Type => (Id::new(&row.kind), row.type_label.clone()),
            Self::ResourceGroup => (
                Id::new(&row.group_id),
                match (&row.group_name, &row.subscription_name) {
                    (Some(name), Some(subscription)) => format!("{name} · {subscription}"),
                    (Some(name), None) => name.clone(),
                    (None, _) => "No resource group".to_owned(),
                },
            ),
            Self::Location => (
                Id::new(&row.location),
                row.location
                    .as_ref()
                    .map_or_else(|| "Unknown location".to_owned(), ToString::to_string),
            ),
            Self::Subscription => (
                Id::new(row.subscription_id),
                row.subscription_name
                    .clone()
                    .unwrap_or_else(|| "No subscription".to_owned()),
            ),
        };
        ReflectedTableGroup {
            id: id.with(self),
            label,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_azure::AzureResourceGroupId;
    use cloud_terrastodon_azure::Resource;
    use cloud_terrastodon_azure::ResourceType;
    use cloud_terrastodon_azure::ScopeImpl;

    #[test]
    fn resource_groups_are_partitioned_by_subscription_and_match_ignoring_case() -> eyre::Result<()>
    {
        let make_row = |subscription: &str, name: &str| {
            let group = AzureResourceGroupId::try_new(subscription, name)?;
            let resource = Resource {
                id: ScopeImpl::ResourceGroup(group.clone()),
                kind:
                    ResourceType::MICROSOFT_DOT_RESOURCES_SLASH_SUBSCRIPTIONS_SLASH_RESOURCEGROUPS,
                name: name.to_owned(),
                location: None,
                tags: Default::default(),
                properties: Default::default(),
            };
            eyre::Ok(ResourceInventoryRow::new(
                &resource,
                Some(group.subscription_id),
                Some("Same subscription name"),
                Some(group),
            ))
        };
        let production = "00000000-0000-0000-0000-000000000001";
        let staging = "00000000-0000-0000-0000-000000000002";
        let first = make_row(production, "Shared")?;
        let alternate_case = make_row(production, "shared")?;
        let other_subscription = make_row(staging, "Shared")?;
        let grouping = ResourceInventoryGrouping::ResourceGroup;

        assert_eq!(
            grouping.group(&first).id,
            grouping.group(&alternate_case).id
        );
        assert_ne!(
            grouping.group(&first).id,
            grouping.group(&other_subscription).id
        );
        let grouping = ResourceInventoryGrouping::Subscription;
        assert_ne!(
            grouping.group(&first).id,
            grouping.group(&other_subscription).id
        );
        Ok(())
    }
}
