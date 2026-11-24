use crate::app::MyApp;
use crate::icons::portal_target_icon;
use crate::management_group_overview::management_group_inventory_ui;
use crate::network_security_group_rules::network_security_group_rules_ui;
use crate::portal_resource_scope::PortalResourceScope;
use crate::portal_resource_scope::network_security_group_id;
use crate::portal_scope_view::PortalScopeView;
use crate::portal_tab_open::PortalTabOpen;
use crate::portal_tab_target::PortalTabTarget;
use crate::reflected_value::ReflectedValue;
use crate::resource_azure_essentials::show_azure_essentials;
use crate::resource_detail_header::resource_detail_header;
use crate::resource_essentials::resource_essentials_ui;
use crate::resource_essentials_format::ResourceEssentialsFormat;
use crate::resource_snapshot::ResourceSnapshot;
use crate::scope_activity::scope_activity_ui;
use crate::scope_iam::scope_iam_ui;
use crate::scope_navigation::scope_navigation;
use crate::scope_tags_ui::scope_tags_ui;
use crate::storage_explorer_ui::storage_explorer_ui;
use crate::storage_networking::storage_networking_ui;
use crate::windows::resources_window::load_resources;
use crate::windows::resources_window::scoped_resources_ui;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_command::app_work::Loadable;
use eframe::egui::CentralPanel;
use eframe::egui::Color32;
use eframe::egui::Frame;
use eframe::egui::Margin;
use eframe::egui::ScrollArea;
use eframe::egui::Stroke;
use eframe::egui::Ui;
use eframe::egui::Vec2;

pub(crate) fn scope_detail_ui(
    app: &mut MyApp,
    ui: &mut Ui,
    scope: &PortalResourceScope,
    view: Option<PortalScopeView>,
) {
    let overview = scope.overview();
    let name = app.portal_tab_title(&overview);
    let inventory = scope.inventory_target(app);
    let icon = portal_target_icon(app, &overview);
    if resource_detail_header(
        ui,
        &name,
        &scope.label(app),
        &app.portal_tab_title(&inventory),
        icon.clone(),
        &inventory,
    )
    .is_some_and(|open| open.apply(app))
    {
        return;
    }
    match scope_navigation(ui, scope, view, &mut app.resource_navigation_expanded, icon) {
        Some(PortalTabOpen::Current(next)) => {
            app.tabs.replace_current(next);
            app.record_portal_visit();
            app.after_history_navigation();
            ui.ctx().request_repaint();
            return;
        }
        Some(open) => {
            open.apply(app);
        }
        None => {}
    }
    CentralPanel::default()
        .frame(Frame::NONE.inner_margin(Margin::symmetric(16, 8)))
        .show(ui, |ui| {
            if let Some(view) = view {
                if app.show_azure_authentication_required(ui) {
                    return;
                }
                match (view, scope) {
                    (PortalScopeView::AccessControl, _) => {
                        scope_iam_ui(app, ui, &scope.arm_scope())
                    }
                    (PortalScopeView::ActivityLog, _) => scope_activity_ui(app, ui, scope),
                    (PortalScopeView::Tags, _) => scope_tags_ui(app, ui, scope),
                    (PortalScopeView::StorageExplorer, PortalResourceScope::StorageAccount(id)) => {
                        storage_explorer_ui(app, ui, id);
                    }
                    (PortalScopeView::Networking, PortalResourceScope::StorageAccount(id)) => {
                        storage_networking_ui(app, ui, id);
                    }
                    (PortalScopeView::StorageExplorer | PortalScopeView::Networking, _) => {
                        ui.label("Select a storage account to use this view.");
                    }
                }
            } else {
                scope_overview_ui(app, ui, scope);
            }
        });
}

fn scope_overview_ui(app: &mut MyApp, ui: &mut Ui, scope: &PortalResourceScope) {
    let arm_scope = scope.arm_scope();
    let resource_id = arm_scope.expanded_form();
    let mut essentials_visible = false;
    let open = resource_essentials_ui(ui, &resource_id, |ui, format, open_output| {
        essentials_visible = true;
        if let Some(response) = open_output
            && let ResourceEssentialsFormat::Cli(format) = format
            && let Some(open) =
                PortalTabOpen::from_response(&response, || PortalTabTarget::ResourceOutput {
                    resource: scope.output_target(),
                    format,
                })
        {
            return Some(open);
        }
        if app.show_azure_authentication_required(ui) {
            return None;
        }
        match format {
            ResourceEssentialsFormat::Azure => {
                ScrollArea::vertical()
                    .max_height(280.0)
                    .show(ui, |ui| show_azure_essentials(app, ui, scope))
                    .inner
            }
            ResourceEssentialsFormat::Tree => {
                let snapshot = ResourceSnapshot::load(app, ui, scope)?;
                ScrollArea::both()
                    .max_height(360.0)
                    .show(ui, |ui| {
                        if let PortalResourceScope::StorageAccount(id) = scope
                            && let Some(inspection) = app.azure.storage_account_browser.entries.get(id)
                        {
                            ui.collapsing("Request", |ui| {
                                ReflectedValue::new(&inspection.request).show(ui)
                            });
                        }
                        snapshot.reflected().show_links(ui, &scope.overview())
                    })
                    .inner
            }
            ResourceEssentialsFormat::Cli(format) => {
                let snapshot = ResourceSnapshot::load(app, ui, scope)?;
                ui.scope(|ui| {
                    ui.set_max_height(ui.available_height().min(340.0));
                    snapshot.show_output(app, ui, format);
                });
                None
            }
        }
    });
    if open.is_some_and(|open| open.apply(app)) {
        return;
    }
    if app.ready_azure_authentication().is_none() {
        return;
    }
    match scope {
        PortalResourceScope::ResourceGroup(_) | PortalResourceScope::Subscription(_) => {
            overview_separator(ui);
            scoped_resources_ui(app, ui, &arm_scope);
        }
        PortalResourceScope::ManagementGroup(id) => {
            overview_separator(ui);
            management_group_inventory_ui(app, ui, id);
        }
        PortalResourceScope::Resource(scope_id) => {
            if let Some(id) = network_security_group_id(scope_id) {
                let snapshot = if essentials_visible {
                    // Essentials already handles the loading state when expanded.
                    app.azure
                        .network_security_groups
                        .get(id)
                        .and_then(Loadable::as_loaded)
                        .cloned()
                        .map(ResourceSnapshot::NetworkSecurityGroup)
                } else {
                    ResourceSnapshot::load(app, ui, scope)
                };
                if let Some(ResourceSnapshot::NetworkSecurityGroup(group)) = snapshot {
                    overview_separator(ui);
                    network_security_group_rules_ui(app, ui, &group);
                }
            } else if matches!(app.azure.resources, Loadable::NotLoaded) {
                load_resources(app, false);
            }
        }
        _ => {}
    }
}

fn overview_separator(ui: &mut Ui) {
    let (_, rect) = ui.allocate_space(Vec2::new(ui.available_width(), 12.0));
    ui.painter().hline(
        rect.x_range(),
        rect.center().y,
        Stroke::new(1.0, Color32::from_rgb(214, 214, 214)),
    );
}
