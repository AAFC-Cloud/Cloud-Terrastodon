use crate::app::MyApp;
use crate::portal_filter_chip::portal_filter_chip;
use crate::portal_search::portal_search;
use crate::reflected_value::ReflectedValue;
use crate::resource_inventory_row::ResourceInventoryRow;
use cloud_terrastodon_azure::AzureActivityLogEvent;
use cloud_terrastodon_azure::AzureActivityLogScope;
use cloud_terrastodon_azure::AzureResourceGroupId;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::ScopeImpl;
use eframe::egui::Id;
use eframe::egui::ScrollArea;
use eframe::egui::Ui;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::sync::Arc;

/// Filters applied to the returned event inventory in the desktop window.
#[derive(Clone, Default)]
pub(crate) struct ScopeActivityFilters {
    scope: Option<ScopeImpl>,
    severity: Option<String>,
    category: Option<String>,
    status: Option<String>,
    scope_search: String,
}

impl ScopeActivityFilters {
    pub fn show(
        &mut self,
        app: &MyApp,
        ui: &mut Ui,
        id: Id,
        requested_scope: &AzureActivityLogScope,
        events: &[Arc<AzureActivityLogEvent>],
    ) {
        ui.horizontal_wrapped(|ui| {
            let scope_label = self
                .scope
                .as_ref()
                .map_or_else(|| "All".to_owned(), |scope| scope_name(app, scope));
            portal_filter_chip(ui, format!("Scope in results · {scope_label}"), |ui| {
                portal_search(ui, &mut self.scope_search, "Find a returned scope", 254.0);
                let search = self.scope_search.trim().to_lowercase();
                if ui
                    .selectable_value(&mut self.scope, None, "All returned scopes")
                    .clicked()
                {
                    ui.close();
                }
                let mut choices = BTreeMap::new();
                for event in events {
                    let scope = event_scope(event, requested_scope);
                    let id = scope.expanded_form();
                    let (subscription, group) = ResourceInventoryRow::ancestors(&id);
                    choices
                        .entry(id.to_ascii_lowercase())
                        .or_insert_with(|| scope.into_owned());
                    if let Some(group) = group {
                        choices
                            .entry(group.expanded_form().to_ascii_lowercase())
                            .or_insert(ScopeImpl::ResourceGroup(group));
                    }
                    if let Some(subscription) = subscription {
                        choices
                            .entry(subscription.expanded_form())
                            .or_insert(ScopeImpl::Subscription(subscription));
                    }
                }
                ScrollArea::vertical()
                    .id_salt(id.with("scopes"))
                    .max_height(260.0)
                    .show(ui, |ui| {
                        for (canonical, scope) in choices {
                            let name = scope_name(app, &scope);
                            if !name.to_lowercase().contains(&search)
                                && !canonical.contains(&search)
                            {
                                continue;
                            }
                            let selected = self.scope.as_ref().is_some_and(|selected| {
                                selected.expanded_form().eq_ignore_ascii_case(&canonical)
                            });
                            if ui
                                .push_id(&canonical, |ui| {
                                    ui.selectable_label(selected, name)
                                        .on_hover_text(scope.expanded_form())
                                        .clicked()
                                })
                                .inner
                            {
                                self.scope = Some(scope);
                                ui.close();
                            }
                        }
                    });
            });
            value_filter(ui, "Severity", &mut self.severity, events, level);
            value_filter(ui, "Category", &mut self.category, events, category);
            value_filter(ui, "Status", &mut self.status, events, status);
            if (self.scope.is_some()
                || self.severity.is_some()
                || self.category.is_some()
                || self.status.is_some())
                && ui.small_button("Clear field filters").clicked()
            {
                *self = Self::default();
            }
        });
    }

    pub fn matching<'a>(
        &'a self,
        events: &'a [Arc<AzureActivityLogEvent>],
        requested_scope: &'a AzureActivityLogScope,
    ) -> impl Iterator<Item = &'a AzureActivityLogEvent> {
        let scope = self
            .scope
            .as_ref()
            .map(|scope| scope.expanded_form().to_ascii_lowercase());
        events.iter().map(Arc::as_ref).filter(move |event| {
            self.severity
                .as_deref()
                .is_none_or(|value| level(event) == Some(value))
                && self
                    .category
                    .as_deref()
                    .is_none_or(|value| category(event) == Some(value))
                && self
                    .status
                    .as_deref()
                    .is_none_or(|value| status(event) == Some(value))
                && scope.as_deref().is_none_or(|prefix| {
                    let id = event_scope(event, requested_scope)
                        .expanded_form()
                        .to_ascii_lowercase();
                    id.strip_prefix(prefix)
                        .is_some_and(|suffix| suffix.is_empty() || suffix.starts_with('/'))
                })
        })
    }
}

fn event_scope<'a>(
    event: &'a AzureActivityLogEvent,
    requested_scope: &'a AzureActivityLogScope,
) -> Cow<'a, ScopeImpl> {
    if let Some(resource_id) = &event.resource_id {
        return Cow::Borrowed(resource_id);
    }
    if !matches!(requested_scope, AzureActivityLogScope::Resource(_))
        && let Some(group) = &event.resource_group_name
        && let Some(subscription) = event
            .subscription_id
            .or_else(|| requested_scope.subscription_id())
    {
        return Cow::Owned(ScopeImpl::ResourceGroup(AzureResourceGroupId::new(
            subscription,
            group.clone(),
        )));
    }
    // The submitted request still identifies the collection when a row omits its resource ID.
    match requested_scope {
        AzureActivityLogScope::Resource(resource) => Cow::Borrowed(resource),
        AzureActivityLogScope::Subscription(subscription) => Cow::Owned(ScopeImpl::Subscription(
            event.subscription_id.unwrap_or(*subscription),
        )),
        AzureActivityLogScope::ResourceGroup(group) => {
            Cow::Owned(ScopeImpl::ResourceGroup(group.clone()))
        }
        AzureActivityLogScope::ManagementGroup(group) => event.subscription_id.map_or_else(
            || Cow::Owned(ScopeImpl::ManagementGroup(group.clone())),
            |subscription| Cow::Owned(ScopeImpl::Subscription(subscription)),
        ),
    }
}

fn scope_name(app: &MyApp, scope: &ScopeImpl) -> String {
    ReflectedValue::new(scope).navigation_target().map_or_else(
        || scope.short_form(),
        |target| app.portal_tab_title(&target).into_owned(),
    )
}

fn value_filter(
    ui: &mut Ui,
    label: &str,
    selected: &mut Option<String>,
    events: &[Arc<AzureActivityLogEvent>],
    field: fn(&AzureActivityLogEvent) -> Option<&str>,
) {
    ui.push_id(label, |ui| {
        portal_filter_chip(
            ui,
            format!("{label} · {}", selected.as_deref().unwrap_or("All")),
            |ui| {
                let options: BTreeSet<_> = events.iter().filter_map(|event| field(event)).collect();
                if ui.selectable_value(selected, None, "All").clicked() {
                    ui.close();
                }
                ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                    for option in options {
                        if ui
                            .selectable_label(selected.as_deref() == Some(option), option)
                            .clicked()
                        {
                            *selected = Some(option.to_owned());
                            ui.close();
                        }
                    }
                });
            },
        );
    });
}

fn level(event: &AzureActivityLogEvent) -> Option<&str> {
    event.level.as_ref().map(|level| level.as_str())
}

fn category(event: &AzureActivityLogEvent) -> Option<&str> {
    event
        .category
        .as_ref()
        .map(|category| category.value.as_str())
}

fn status(event: &AzureActivityLogEvent) -> Option<&str> {
    event.status.as_ref().map(|status| status.value.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_azure::AzureActivityLogEventLevel;
    use cloud_terrastodon_azure::AzureActivityLogLocalizableString;

    #[test]
    fn combines_scope_and_severity_for_loaded_events() -> eyre::Result<()> {
        let subscription = "00000000-0000-0000-0000-000000000001";
        let group = AzureResourceGroupId::try_new(subscription, "internal")?;
        let request = AzureActivityLogScope::Subscription(group.subscription_id);
        let event = |group_name: &str, level: AzureActivityLogEventLevel| -> eyre::Result<_> {
            let resource = format!(
                "/subscriptions/{subscription}/resourceGroups/{group_name}/providers/Example.Widgets/widgets/sample"
            );
            Ok(Arc::new(AzureActivityLogEvent {
                event_data_id: resource.clone(),
                event_timestamp: "2026-01-01T00:00:00Z".parse()?,
                operation_name: AzureActivityLogLocalizableString {
                    value: "Write".into(),
                    localized_value: None,
                },
                category: None,
                status: None,
                resource_id: Some(resource.parse()?),
                level: Some(level),
                resource_group_name: None,
                subscription_id: None,
            }))
        };
        let events = [
            event("INTERNAL", AzureActivityLogEventLevel::Warning)?,
            event("internal-reporting", AzureActivityLogEventLevel::Warning)?,
            event("internal", AzureActivityLogEventLevel::Informational)?,
        ];
        let filters = ScopeActivityFilters {
            scope: Some(ScopeImpl::ResourceGroup(group)),
            severity: Some("Warning".into()),
            ..Default::default()
        };
        let matching: Vec<_> = filters.matching(&events, &request).collect();
        assert_eq!(matching.len(), 1);
        assert_eq!(matching[0].event_data_id, events[0].event_data_id);
        Ok(())
    }
}
