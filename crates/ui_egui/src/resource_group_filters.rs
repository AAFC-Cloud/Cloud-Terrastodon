use cloud_terrastodon_azure::AzureLocationName;
use cloud_terrastodon_azure::AzureResourceGroup;
use cloud_terrastodon_azure::SubscriptionId;
use cloud_terrastodon_azure::SubscriptionName;
use eframe::egui::ScrollArea;
use eframe::egui::TextEdit;
use eframe::egui::Ui;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

/// Predicates over the returned resource groups. Checked values of one field
/// are alternatives, and predicates for different fields are combined.
#[derive(Default)]
pub(crate) struct ResourceGroupFilters {
    name: String,
    subscriptions: BTreeMap<SubscriptionId, SubscriptionName>,
    locations: BTreeSet<AzureLocationName>,
}

impl ResourceGroupFilters {
    pub fn show<'a>(
        &mut self,
        ui: &mut Ui,
        resource_groups: impl Iterator<Item = &'a AzureResourceGroup> + Clone,
    ) -> bool {
        let mut changed = false;
        ui.horizontal_wrapped(|ui| {
            changed |= ui
                .add(TextEdit::singleline(&mut self.name).hint_text("Filter by name"))
                .changed();
            ui.menu_button("Add filter", |ui| {
                ui.menu_button("Subscription", |ui| {
                    let subscriptions: BTreeMap<_, _> = resource_groups
                        .clone()
                        .map(|group| (group.id.subscription_id, &group.subscription_name))
                        .collect();
                    let mut choices: Vec<_> = subscriptions.into_iter().collect();
                    choices.sort_unstable_by_key(|(id, name)| (*name, *id));
                    ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
                        for (id, name) in choices {
                            ui.push_id(id, |ui| {
                                let mut selected = self.subscriptions.contains_key(&id);
                                if ui
                                    .checkbox(&mut selected, name.as_str())
                                    .on_hover_text(id.to_string())
                                    .changed()
                                {
                                    changed = true;
                                    if selected {
                                        self.subscriptions.insert(id, name.clone());
                                    } else {
                                        self.subscriptions.remove(&id);
                                    }
                                }
                            });
                        }
                    });
                });
                ui.menu_button("Location", |ui| {
                    let locations: BTreeSet<_> = resource_groups
                        .clone()
                        .map(|group| &group.location)
                        .collect();
                    ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
                        for location in locations {
                            let mut selected = self.locations.contains(location);
                            if ui.checkbox(&mut selected, location.to_string()).changed() {
                                changed = true;
                                if selected {
                                    self.locations.insert(location.clone());
                                } else {
                                    self.locations.remove(location);
                                }
                            }
                        }
                    });
                });
            });
            if (!self.name.is_empty()
                || !self.subscriptions.is_empty()
                || !self.locations.is_empty())
                && ui.button("Clear filters").clicked()
            {
                *self = Self::default();
                changed = true;
            }
        });
        if !self.subscriptions.is_empty() || !self.locations.is_empty() {
            ui.horizontal_wrapped(|ui| {
                self.subscriptions.retain(|id, name| {
                    let remove = ui
                        .push_id(id, |ui| {
                            ui.small_button(format!("Subscription · {name} ×"))
                                .on_hover_text(format!("Remove filter for {id}"))
                                .clicked()
                        })
                        .inner;
                    changed |= remove;
                    !remove
                });
                self.locations.retain(|location| {
                    let remove = ui
                        .small_button(format!("Location · {location} ×"))
                        .clicked();
                    changed |= remove;
                    !remove
                });
            });
        }
        changed
    }

    pub fn filter<'a>(
        &self,
        resource_groups: impl Iterator<Item = &'a AzureResourceGroup>,
    ) -> impl Iterator<Item = &'a AzureResourceGroup> {
        let name = self.name.trim().to_lowercase();
        resource_groups.filter(move |group| {
            (self.subscriptions.is_empty()
                || self.subscriptions.contains_key(&group.id.subscription_id))
                && (self.locations.is_empty() || self.locations.contains(&group.location))
                && (name.is_empty() || group.name.to_lowercase().contains(&name))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reflected_value::ReflectedValue;
    use cloud_terrastodon_azure::AzureResourceGroupId;
    use cloud_terrastodon_azure::AzureResourceGroupName;

    #[test]
    fn reflected_columns_and_filters_use_resource_group_response_fields() -> eyre::Result<()> {
        let location: AzureLocationName = "canadacentral".parse()?;
        let group = |subscription_id: &str, name: &str, group_location: &str| -> eyre::Result<_> {
            let name: AzureResourceGroupName = name.parse()?;
            Ok(AzureResourceGroup {
                id: AzureResourceGroupId::new(
                    subscription_id.parse::<SubscriptionId>()?,
                    name.clone(),
                ),
                name,
                subscription_name: "Synthetic subscription".parse()?,
                tenant_id: "00000000-0000-0000-0000-000000000003".parse()?,
                location: group_location.parse()?,
                managed_by: None,
                properties: Default::default(),
                tags: Default::default(),
            })
        };
        let a = "00000000-0000-0000-0000-000000000001";
        let b = "00000000-0000-0000-0000-000000000002";
        let groups = [
            group(a, "sample-alpha", "canadacentral")?,
            group(a, "sample-beta", "eastus")?,
            group(b, "sample-gamma", "canadacentral")?,
            group(b, "another-name", "canadacentral")?,
        ];
        let mut filters = ResourceGroupFilters {
            name: "SAMPLE".to_owned(),
            locations: BTreeSet::from([location]),
            ..Default::default()
        };
        filters
            .subscriptions
            .insert(a.parse()?, groups[0].subscription_name.clone());
        assert_eq!(filters.filter(groups.iter()).count(), 1);
        filters
            .subscriptions
            .insert(b.parse()?, groups[2].subscription_name.clone());
        let names: Vec<_> = filters
            .filter(groups.iter())
            .filter_map(|group| ReflectedValue::new(group).column("name")?.table_text())
            .collect();
        assert_eq!(names, ["sample-alpha", "sample-gamma"]);
        let row = ReflectedValue::new(&groups[0]);
        assert_eq!(
            row.column("subscription_name")
                .and_then(ReflectedValue::table_text)
                .as_deref(),
            Some("Synthetic subscription")
        );
        assert_eq!(
            row.column("location").and_then(ReflectedValue::table_text),
            Some(groups[0].location.to_string())
        );
        Ok(())
    }
}
