use crate::icons::resource_icon;
use crate::reflected_value::ReflectedValue;
use cloud_terrastodon_azure::AzureLocationName;
use cloud_terrastodon_azure::AzureResourceGroupId;
use cloud_terrastodon_azure::Resource;
use cloud_terrastodon_azure::ResourceType;
use cloud_terrastodon_azure::ScopeImpl;
use cloud_terrastodon_azure::SubscriptionId;
use cloud_terrastodon_iconography_azure::icon_for_resource_type;
use eframe::egui::ImageSource;
use facet::Facet;
use std::str::Split;

/// The user-facing labels and exact ARM identities of an inventory result.
#[derive(Facet)]
pub(crate) struct ResourceInventoryRow {
    pub id: ScopeImpl,
    pub name: String,
    pub kind: ResourceType,
    pub type_label: String,
    pub location: Option<AzureLocationName>,
    pub group_id: Option<AzureResourceGroupId>,
    pub group_name: Option<String>,
    pub subscription_id: Option<SubscriptionId>,
    pub subscription_name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_azure::Scope;

    #[test]
    fn scoped_inventory_excludes_containers_and_uses_exact_typed_ancestry() -> eyre::Result<()> {
        let subscription = "00000000-0000-0000-0000-000000000001";
        let group = AzureResourceGroupId::try_new(subscription, "synthetic")?;
        let scope = ScopeImpl::ResourceGroup(group.clone());
        for (owner, name, expected) in [
            (subscription, "SYNTHETIC", true),
            (subscription, "synthetic-extra", false),
            ("00000000-0000-0000-0000-000000000002", "synthetic", false),
        ] {
            let child = format!(
                "/SUBSCRIPTIONS/{owner}/RESOURCEGROUPS/{name}/providers/Example.Widget/widgets/sample"
            );
            let id = ScopeImpl::try_from_expanded(&child)?;
            let (subscription, group) = ResourceInventoryRow::ancestors(&child);
            assert_eq!(
                ResourceInventoryRow::belongs_to(&id, subscription, group.as_ref(), &scope),
                expected
            );
        }
        assert!(!ResourceInventoryRow::belongs_to(
            &scope,
            Some(group.subscription_id),
            Some(&group),
            &scope
        ));
        Ok(())
    }
}

impl ResourceInventoryRow {
    pub fn new(
        resource: &Resource,
        subscription_id: Option<SubscriptionId>,
        subscription_name: Option<&str>,
        group_id: Option<AzureResourceGroupId>,
    ) -> Self {
        let name = match &resource.id {
            ScopeImpl::ResourceGroup(id) => id.resource_group_name.as_str(),
            _ => resource.display_name().unwrap_or(&resource.name),
        };
        let type_label = icon_for_resource_type(&resource.kind).map_or_else(
            || resource.kind.to_string(),
            |icon| icon.name().into_owned(),
        );
        Self {
            id: resource.id.clone(),
            name: name.to_owned(),
            kind: resource.kind.clone(),
            type_label,
            location: resource.location.clone(),
            group_name: group_id
                .as_ref()
                .map(|id| id.resource_group_name.to_string()),
            group_id,
            subscription_id,
            subscription_name: subscription_id
                .map(|id| subscription_name.map_or_else(|| id.to_string(), str::to_owned)),
        }
    }

    pub fn icon(row: ReflectedValue<'_>) -> Option<ImageSource<'static>> {
        row.field("kind")?.get::<ResourceType>().map(resource_icon)
    }

    pub fn belongs_to(
        id: &ScopeImpl,
        subscription: Option<SubscriptionId>,
        group: Option<&AzureResourceGroupId>,
        scope: &ScopeImpl,
    ) -> bool {
        if matches!(id, ScopeImpl::ResourceGroup(_) | ScopeImpl::Subscription(_)) {
            return false;
        }
        match scope {
            ScopeImpl::ResourceGroup(expected) => group == Some(expected),
            ScopeImpl::Subscription(expected) => subscription == Some(*expected),
            _ => false,
        }
    }

    /// Extract typed ancestors even when the resource itself has no detail page.
    pub fn ancestors(id: &str) -> (Option<SubscriptionId>, Option<AzureResourceGroupId>) {
        let Some((subscription_id, mut parts)) = Self::subscription(id) else {
            return (None, None);
        };
        let group = match (parts.next(), parts.next()) {
            (Some(segment), Some(name)) if segment.eq_ignore_ascii_case("resourceGroups") => {
                AzureResourceGroupId::try_new(subscription_id, name).ok()
            }
            _ => None,
        };
        (Some(subscription_id), group)
    }

    fn subscription(id: &str) -> Option<(SubscriptionId, Split<'_, char>)> {
        let mut parts = id.trim_start_matches('/').split('/');
        if !parts.next()?.eq_ignore_ascii_case("subscriptions") {
            return None;
        }
        Some((parts.next()?.parse().ok()?, parts))
    }
}
