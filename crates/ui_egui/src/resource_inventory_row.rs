use crate::app::MyApp;
use crate::icons::resource_icon;
use crate::portal_azure_content::PortalAzureContent;
use crate::reflected_value::ReflectedValue;
use cloud_terrastodon_azure::AzureLocationName;
use cloud_terrastodon_azure::AzureResourceGroupId;
use cloud_terrastodon_azure::Resource;
use cloud_terrastodon_azure::ResourceType;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::ScopeImpl;
use cloud_terrastodon_azure::SubscriptionId;
use cloud_terrastodon_iconography_azure::icon_for_resource_type;
use eframe::egui::ImageSource;
use eframe::egui::Ui;
use facet::Facet;
use std::str::Split;
use std::sync::Arc;

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
    use crate::portal_tab_target::PortalTabTarget;

    #[test]
    fn inventory_name_can_link_to_a_nested_resource() {
        let id = ScopeImpl::from(concat!(
            "/subscriptions/00000000-0000-0000-0000-000000000001",
            "/resourceGroups/synthetic",
            "/providers/Microsoft.Compute/galleries/gallery/images/image/versions/1.0.0",
        ));
        let resource = Resource {
            id: id.clone(),
            kind: ResourceType::MICROSOFT_DOT_COMPUTE_SLASH_GALLERIES_SLASH_IMAGES_SLASH_VERSIONS,
            name: "1.0.0".to_owned(),
            location: None,
            tags: Default::default(),
            properties: Default::default(),
        };
        let (subscription, group) = ResourceInventoryRow::ancestors(&resource.id.expanded_form());
        let row = ResourceInventoryRow::new(&resource, subscription, None, group);
        assert_eq!(
            ReflectedValue::new(&row)
                .column("id")
                .and_then(ReflectedValue::navigation_target),
            Some(PortalTabTarget::Resource(Box::new(id)))
        );
    }

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

impl MyApp {
    pub(crate) fn inventory_resource(&self, id: &ScopeImpl) -> Option<&Arc<Resource>> {
        self.azure.inventory_resource(id)
    }
}

impl PortalAzureContent {
    pub(crate) fn inventory_resource(&self, id: &ScopeImpl) -> Option<&Arc<Resource>> {
        let resources = self.resources.as_loaded()?;
        if let Some(resource) = resources.iter().find(|resource| resource.id == *id) {
            return Some(resource);
        }
        // The same resource may be linked using different casing in other ARM responses.
        let id = id.expanded_form();
        resources
            .iter()
            .find(|resource| resource.id.expanded_form().eq_ignore_ascii_case(&id))
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

    pub fn icon(_ui: &Ui, row: ReflectedValue<'_>) -> Option<ImageSource<'static>> {
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

    /// Extract the subscription and resource group from an ARM ID's path.
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
