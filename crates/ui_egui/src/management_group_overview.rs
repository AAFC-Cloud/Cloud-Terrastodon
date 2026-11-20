use crate::app::MyApp;
use crate::icons::page_icon;
use crate::management_group_hierarchy::ManagementGroupHierarchy;
use crate::management_group_tree::show_management_group_tree;
use crate::portal_page::PortalPage;
use crate::portal_tab_target::PortalTabTarget;
use crate::reflected_value::ReflectedValue;
use crate::resource_detail_header::resource_detail_header;
use crate::scope_navigation::navigation_tile;
use crate::sidebar_expansion_toggle::sidebar_expansion_toggle;
use crate::windows::management_groups_window::load_management_groups;
use crate::workers::load_subscriptions::load_subscriptions;
use cloud_terrastodon_azure::ManagementGroupId;
use cloud_terrastodon_command::app_work::Loadable;
use eframe::egui::CentralPanel;
use eframe::egui::CollapsingHeader;
use eframe::egui::Color32;
use eframe::egui::Frame;
use eframe::egui::Grid;
use eframe::egui::Margin;
use eframe::egui::Panel;
use eframe::egui::ScrollArea;
use eframe::egui::Ui;
use teamy_tracing::gui::DiagnosticReportView;

pub(crate) fn management_group_details_ui(app: &mut MyApp, ui: &mut Ui, id: &ManagementGroupId) {
    if resource_detail_header(
        ui,
        app.management_group_title(id).unwrap_or(id.name()),
        "Management group",
        PortalPage::ManagementGroups,
    ) {
        app.navigate(PortalPage::ManagementGroups);
        return;
    }
    if app.show_azure_authentication_required(ui) {
        return;
    }
    if matches!(app.management_groups, Loadable::NotLoaded) {
        load_management_groups(app, false);
    }
    if matches!(app.subscriptions, Loadable::NotLoaded) {
        load_subscriptions(app);
    }

    let view_id = ui.make_persistent_id(("management-group-view", app.account_generation, id));
    let mut subscriptions_only: bool = ui.data(|data| data.get_temp(view_id)).unwrap_or_default();
    let show_labels = app.resource_navigation_expanded;
    Panel::left("management_group_navigation")
        .exact_size(if show_labels { 174.0 } else { 44.0 })
        .resizable(false)
        .frame(Frame::new().fill(Color32::WHITE))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            sidebar_expansion_toggle(
                ui,
                &mut app.resource_navigation_expanded,
                "resource navigation",
            );
            if navigation_tile(
                ui,
                "Overview",
                page_icon(PortalPage::ManagementGroups),
                !subscriptions_only,
                show_labels,
            ) {
                subscriptions_only = false;
            }
            if navigation_tile(
                ui,
                "Subscriptions",
                page_icon(PortalPage::Subscriptions),
                subscriptions_only,
                show_labels,
            ) {
                subscriptions_only = true;
            }
        });
    ui.data_mut(|data| data.insert_temp(view_id, subscriptions_only));

    let next = CentralPanel::default()
        .frame(Frame::NONE.inner_margin(Margin::symmetric(12, 8)))
        .show(ui, |ui| {
            show_inventory_status(app, ui);
            let groups = app.management_groups.as_loaded();
            let subscriptions = app.subscriptions.as_loaded();
            let hierarchy = ManagementGroupHierarchy::new(
                groups.map(Vec::as_slice).unwrap_or_default(),
                subscriptions
                    .map(|subscriptions| subscriptions.as_slice())
                    .unwrap_or_default(),
            );
            ui.push_id(view_id, |ui| {
                if !subscriptions_only {
                    if let Some(next) = show_essentials(
                        ui,
                        id,
                        hierarchy
                            .group(id)
                            .map_or(id.name(), |group| group.display_name.as_str()),
                        &hierarchy,
                        groups.is_some() || subscriptions.is_some(),
                        subscriptions.is_some(),
                    ) {
                        return Some(next);
                    }
                    if let Some(group) = app.management_group(id) {
                        ui.collapsing("Group properties", |ui| {
                            ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                                ReflectedValue::new(group).show(ui);
                            });
                        });
                    }
                    ui.separator();
                }
                if subscriptions.is_some() || (!subscriptions_only && groups.is_some()) {
                    show_management_group_tree(
                        ui,
                        &hierarchy,
                        id,
                        subscriptions_only,
                        subscriptions.is_some(),
                    )
                } else {
                    None
                }
            })
            .inner
        })
        .inner;
    if let Some(next) = next {
        app.navigate_tab(next);
    }
}

fn show_inventory_status(app: &mut MyApp, ui: &mut Ui) {
    if let Loadable::Failed { error, .. } = &app.management_groups {
        ui.collapsing("Could not load the management group inventory", |ui| {
            DiagnosticReportView::new(error).show(ui);
        });
        if ui.small_button("Retry management groups").clicked() {
            load_management_groups(app, false);
        }
    }
    if let Loadable::Failed { error, .. } = &app.subscriptions {
        ui.collapsing("Could not load the subscription inventory", |ui| {
            DiagnosticReportView::new(error).show(ui);
        });
        if ui.small_button("Retry subscriptions").clicked() {
            load_subscriptions(app);
        }
    }
    if matches!(app.management_groups, Loadable::Loading { .. })
        || matches!(app.subscriptions, Loadable::Loading { .. })
    {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label("Loading group hierarchy…");
        });
    }
}

fn show_essentials(
    ui: &mut Ui,
    id: &ManagementGroupId,
    title: &str,
    hierarchy: &ManagementGroupHierarchy<'_>,
    child_groups_available: bool,
    subscriptions_available: bool,
) -> Option<PortalTabTarget> {
    let ancestors = hierarchy.ancestors(id);
    let mut next = None;
    CollapsingHeader::new("Essentials")
        .default_open(true)
        .show(ui, |ui| {
            Grid::new("management_group_essentials")
                .num_columns(2)
                .spacing([16.0, 5.0])
                .show(ui, |ui| {
                    ui.label("Name");
                    ui.label(title);
                    ui.end_row();

                    ui.label("Group ID");
                    ui.horizontal_wrapped(|ui| {
                        ui.monospace(id.name());
                        if ui.small_button("Copy resource ID").clicked() {
                            ui.ctx().copy_text(id.to_string());
                        }
                    });
                    ui.end_row();

                    ui.label("Parent group");
                    if let Some(parent) = ancestors.last() {
                        if ui.link(&parent.display_name).clicked() {
                            next = Some(PortalTabTarget::ManagementGroup(parent.id()));
                        }
                    } else {
                        ui.weak("—");
                    }
                    ui.end_row();

                    ui.label("Path");
                    ui.horizontal_wrapped(|ui| {
                        for ancestor in ancestors {
                            if ui.link(&ancestor.display_name).clicked() {
                                next = Some(PortalTabTarget::ManagementGroup(ancestor.id()));
                            }
                            ui.label("›");
                        }
                        ui.label(title);
                    });
                    ui.end_row();

                    ui.label("Visible child groups");
                    if child_groups_available {
                        ui.label(hierarchy.child_groups(id).count().to_string());
                    } else {
                        ui.weak("—");
                    }
                    ui.end_row();

                    ui.label("Visible subscriptions");
                    if subscriptions_available {
                        ui.label(hierarchy.subscription_count(id).to_string());
                    } else {
                        ui.weak("—");
                    }
                    ui.end_row();
                });
        });
    next
}
