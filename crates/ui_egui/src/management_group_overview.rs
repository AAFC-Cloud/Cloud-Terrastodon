use crate::app::MyApp;
use crate::management_group_hierarchy::ManagementGroupHierarchy;
use crate::management_group_tree::show_management_group_tree;
use crate::portal_tab_open::PortalTabOpen;
use crate::portal_tab_target::PortalTabTarget;
use crate::windows::management_groups_window::load_management_groups;
use crate::workers::load_subscriptions::load_subscriptions;
use cloud_terrastodon_azure::ManagementGroupId;
use cloud_terrastodon_command::app_work::Loadable;
use eframe::egui::Grid;
use eframe::egui::Ui;
use teamy_tracing::gui::DiagnosticReportView;

/// Show the child hierarchy and subscriptions below the shared resource essentials.
pub(crate) fn management_group_inventory_ui(app: &mut MyApp, ui: &mut Ui, id: &ManagementGroupId) {
    if matches!(app.azure.management_groups, Loadable::NotLoaded) {
        load_management_groups(app, false);
    }
    if matches!(app.azure.subscriptions, Loadable::NotLoaded) {
        load_subscriptions(app);
    }

    let view_id =
        ui.make_persistent_id(("management-group-view", app.azure.account_generation, id));
    let mut subscriptions_only: bool = ui.data(|data| data.get_temp(view_id)).unwrap_or_default();
    let changed = ui
        .horizontal(|ui| {
            let hierarchy = ui.selectable_value(&mut subscriptions_only, false, "Hierarchy");
            let subscriptions = ui.selectable_value(&mut subscriptions_only, true, "Subscriptions");
            hierarchy.changed() || subscriptions.changed()
        })
        .inner;
    if changed {
        ui.data_mut(|data| data.insert_temp(view_id, subscriptions_only));
    }
    show_inventory_status(app, ui);

    let subscriptions_available = app.azure.subscriptions.as_loaded().is_some();
    if !subscriptions_available
        && (subscriptions_only || app.azure.management_groups.as_loaded().is_none())
    {
        return;
    }
    let hierarchy = loaded_hierarchy(app);
    let next = ui
        .push_id(view_id, |ui| {
            show_management_group_tree(
                ui,
                &hierarchy,
                id,
                subscriptions_only,
                subscriptions_available,
            )
        })
        .inner;
    if let Some(next) = next {
        next.apply(app);
    }
}

fn loaded_hierarchy(app: &MyApp) -> ManagementGroupHierarchy<'_> {
    ManagementGroupHierarchy::new(
        app.azure
            .management_groups
            .as_loaded()
            .map(Vec::as_slice)
            .unwrap_or_default(),
        app.azure
            .subscriptions
            .as_loaded()
            .map(|subscriptions| subscriptions.as_slice())
            .unwrap_or_default(),
    )
}

fn show_inventory_status(app: &mut MyApp, ui: &mut Ui) {
    if let Loadable::Failed { error, .. } = &app.azure.management_groups {
        ui.collapsing("Could not load the management group inventory", |ui| {
            DiagnosticReportView::new(error).show(ui);
        });
        if ui.small_button("Retry management groups").clicked() {
            load_management_groups(app, false);
        }
    }
    if let Loadable::Failed { error, .. } = &app.azure.subscriptions {
        ui.collapsing("Could not load the subscription inventory", |ui| {
            DiagnosticReportView::new(error).show(ui);
        });
        if ui.small_button("Retry subscriptions").clicked() {
            load_subscriptions(app);
        }
    }
    if matches!(app.azure.management_groups, Loadable::Loading { .. })
        || matches!(app.azure.subscriptions, Loadable::Loading { .. })
    {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label("Loading group hierarchy…");
        });
    }
}

/// Add management group fields to the Azure Essentials representation.
pub(crate) fn management_group_essentials(
    app: &MyApp,
    ui: &mut Ui,
    id: &ManagementGroupId,
) -> Option<PortalTabOpen> {
    let hierarchy = loaded_hierarchy(app);
    let title = hierarchy
        .group(id)
        .map_or(id.name(), |group| group.display_name.as_str());
    let ancestors = hierarchy.ancestors(id);
    let subscriptions_available = app.azure.subscriptions.as_loaded().is_some();
    let groups_available = app.azure.management_groups.as_loaded().is_some();
    let mut next = None;
    Grid::new(("management_group_essentials", id))
        .num_columns(2)
        .spacing([16.0, 5.0])
        .show(ui, |ui| {
            ui.label("Group ID");
            ui.monospace(id.name());
            ui.end_row();

            ui.label("Parent group");
            if let Some(parent) = ancestors.last() {
                next = PortalTabOpen::from_response(&ui.link(&parent.display_name), || {
                    PortalTabTarget::ManagementGroup(parent.id())
                });
            } else {
                ui.weak("—");
            }
            ui.end_row();

            ui.label("Path");
            ui.horizontal_wrapped(|ui| {
                for ancestor in ancestors {
                    if let Some(action) =
                        PortalTabOpen::from_response(&ui.link(&ancestor.display_name), || {
                            PortalTabTarget::ManagementGroup(ancestor.id())
                        })
                    {
                        next = Some(action);
                    }
                    ui.label("›");
                }
                ui.label(title);
            });
            ui.end_row();

            ui.label("Visible child groups");
            if groups_available || subscriptions_available {
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
    next
}
