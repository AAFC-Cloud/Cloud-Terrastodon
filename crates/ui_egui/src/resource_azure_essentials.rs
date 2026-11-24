use crate::app::MyApp;
use crate::management_group_overview::management_group_essentials;
use crate::portal_resource_scope::PortalResourceScope;
use crate::portal_tab_open::PortalTabOpen;
use crate::portal_tab_target::PortalTabTarget;
use crate::resource_inventory_row::ResourceInventoryRow;
use crate::resource_snapshot::ResourceSnapshot;
use cloud_terrastodon_azure::Scope;
use eframe::egui::Align2;
use eframe::egui::Atom;
use eframe::egui::AtomLayout;
use eframe::egui::Color32;
use eframe::egui::Frame;
use eframe::egui::Grid;
use eframe::egui::Margin;
use eframe::egui::TextWrapMode;
use eframe::egui::Ui;
use eframe::egui::WidgetText;
use std::collections::HashMap;

pub(crate) fn show_azure_essentials(
    app: &mut MyApp,
    ui: &mut Ui,
    scope: &PortalResourceScope,
) -> Option<PortalTabOpen> {
    if let PortalResourceScope::ManagementGroup(id) = scope {
        return management_group_essentials(app, ui, id);
    }
    let snapshot = ResourceSnapshot::load(app, ui, scope)?;

    Grid::new("resource-essentials-properties")
        .num_columns(2)
        .spacing([22.0, 6.0])
        .show(ui, |ui| {
            let mut destination = None;
            match &snapshot {
                ResourceSnapshot::Resource(resource) => {
                    let (subscription, group) =
                        ResourceInventoryRow::ancestors(&resource.id.expanded_form());
                    if let Some(group) = group {
                        destination = property_link(
                            app,
                            ui,
                            "Resource group",
                            PortalTabTarget::ResourceGroup(group),
                        );
                    }
                    if let Some(id) = subscription {
                        destination = property_link(
                            app,
                            ui,
                            "Subscription",
                            PortalTabTarget::Subscription(id),
                        )
                        .or(destination);
                    }
                    if let Some(location) = &resource.location {
                        property(ui, "Location", location.to_string());
                    }
                    if let Some(state) = resource.properties.provisioning_state.as_deref() {
                        property(ui, "Provisioning state", state);
                    }
                    show_tags(ui, &resource.tags);
                }
                ResourceSnapshot::ResourceGroup(group) => {
                    ui.label("Subscription");
                    destination = PortalTabOpen::from_response(
                        &ui.link(group.subscription_name.as_str()),
                        || PortalTabTarget::Subscription(group.id.subscription_id),
                    );
                    ui.end_row();
                    property(ui, "Location", group.location.to_string());
                    show_tags(ui, &group.tags);
                }
                ResourceSnapshot::Subscription(subscription) => {
                    property(ui, "Tenant ID", subscription.tenant_id.to_string());
                    if let Some(group) = subscription.management_group_ancestors_chain.first() {
                        destination = property_link(
                            app,
                            ui,
                            "Parent management group",
                            PortalTabTarget::ManagementGroup(group.id()),
                        );
                    }
                    show_tags(ui, &subscription.tags);
                }
                ResourceSnapshot::StorageAccount(account) => {
                    let group = &account.id.resource_group_id;
                    destination = property_link(
                        app,
                        ui,
                        "Resource group",
                        PortalTabTarget::ResourceGroup(group.clone()),
                    );
                    destination = property_link(
                        app,
                        ui,
                        "Subscription",
                        PortalTabTarget::Subscription(group.subscription_id),
                    )
                    .or(destination);
                    property(ui, "Location", &account.location);
                    property(ui, "SKU", &account.sku.name);
                    property(ui, "Account kind", format!("{:?}", account.kind));
                    if let Some(state) = &account.properties.provisioning_state {
                        property(ui, "Provisioning state", String::from(state));
                    }
                    if let Some(allowed) = account.properties.allow_blob_public_access {
                        property(
                            ui,
                            "Allows blob anonymous access",
                            if allowed { "Yes" } else { "No" },
                        );
                    }
                    show_tags(ui, &account.tags);
                }
                ResourceSnapshot::NetworkSecurityGroup(group) => {
                    let resource_group_id = &group.id.resource_group_id;
                    destination = property_link(
                        app,
                        ui,
                        "Resource group",
                        PortalTabTarget::ResourceGroup(resource_group_id.clone()),
                    );
                    destination = property_link(
                        app,
                        ui,
                        "Subscription",
                        PortalTabTarget::Subscription(resource_group_id.subscription_id),
                    )
                    .or(destination);
                    property(ui, "Location", group.location.to_string());
                    if let Some(state) = &group.properties.provisioning_state {
                        property(ui, "Provisioning state", String::from(state));
                    }
                    show_tags(ui, &group.tags);
                }
                ResourceSnapshot::ManagementGroup(_) => {}
            }
            destination
        })
        .inner
}

fn property(ui: &mut Ui, label: &str, value: impl Into<WidgetText>) {
    ui.label(label);
    ui.label(value);
    ui.end_row();
}

fn property_link(
    app: &MyApp,
    ui: &mut Ui,
    label: &str,
    target: PortalTabTarget,
) -> Option<PortalTabOpen> {
    ui.label(label);
    let response = ui.link(app.portal_tab_title(&target).as_ref());
    ui.end_row();
    PortalTabOpen::from_response(&response, || target)
}

fn show_tags(ui: &mut Ui, tags: &HashMap<String, String>) {
    if tags.is_empty() {
        return;
    }
    let mut tags: Vec<_> = tags.iter().collect();
    tags.sort_unstable_by_key(|(key, _)| *key);
    ui.label("Tags");
    let segment_width = (ui.available_width() * 0.5).clamp(12.0, 252.0);
    ui.horizontal_wrapped(|ui| {
        for (key, value) in tags {
            let segment = |text: &str, color| {
                Atom::layout(
                    AtomLayout::new(text)
                        .frame(
                            Frame::new()
                                .fill(color)
                                .inner_margin(Margin::symmetric(6, 2)),
                        )
                        .align2(Align2::LEFT_CENTER)
                        .max_width(segment_width)
                        .wrap_mode(TextWrapMode::Truncate),
                )
            };
            AtomLayout::new((
                segment(key, Color32::from_rgb(225, 239, 248)),
                segment(value, Color32::from_rgb(242, 246, 249)),
            ))
            .gap(0.0)
            .allocate(ui)
            .paint(ui)
            .response
            .on_hover_ui(|ui| {
                ui.label(key);
                ui.label(value);
            });
        }
    });
    ui.end_row();
}
