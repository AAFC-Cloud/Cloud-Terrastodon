use crate::app::MyApp;
use crate::portal_resource_output_target::PortalResourceOutputTarget;
use crate::portal_resource_scope::PortalResourceScope;
use crate::portal_scope_view::PortalScopeView;
use crate::portal_tab_target::PortalTabTarget;
use crate::reflected_value::ReflectedValue;
use crate::resource_detail_header::resource_detail_header_with_output;
use crate::scope_activity::scope_activity_ui;
use crate::scope_iam::scope_iam_ui;
use crate::scope_navigation::scope_navigation;
use crate::windows::resources_window::scoped_resources_ui;
use crate::workers::load_resource_groups::load_resource_groups;
use crate::workers::load_subscriptions::load_subscriptions;
use cloud_terrastodon_app::OutputFormat;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_command::app_work::Loadable;
use eframe::egui::CentralPanel;
use eframe::egui::Frame;
use eframe::egui::ScrollArea;
use eframe::egui::Ui;
use teamy_tracing::gui::DiagnosticReportView;

pub(crate) fn scope_detail_ui(
    app: &mut MyApp,
    ui: &mut Ui,
    scope: &PortalResourceScope,
    view: Option<PortalScopeView>,
) {
    let overview = scope.overview();
    let name = app.portal_tab_title(&overview);
    let (inventory, view_output) =
        resource_detail_header_with_output(ui, &name, scope.label(), scope.inventory());
    if inventory {
        app.navigate(scope.inventory());
        return;
    }
    if view_output {
        app.navigate_tab(PortalTabTarget::ResourceOutput {
            resource: PortalResourceOutputTarget::Scope(scope.clone()),
            format: OutputFormat::Json,
        });
        return;
    }
    if let Some(next) = scope_navigation(ui, scope, view, &mut app.resource_navigation_expanded) {
        app.tabs.replace_current(next);
        app.after_history_navigation();
        ui.ctx().request_repaint();
        return;
    }
    CentralPanel::default().frame(Frame::NONE).show(ui, |ui| {
        if app.show_azure_authentication_required(ui) {
            return;
        }
        match view {
            Some(PortalScopeView::AccessControl) => scope_iam_ui(app, ui, &scope.arm_scope()),
            Some(PortalScopeView::ActivityLog) => scope_activity_ui(app, ui, scope),
            None => {
                scope_properties(app, ui, scope);
                ui.add_space(12.0);
                scoped_resources_ui(app, ui, &scope.arm_scope());
            }
        }
    });
}

fn scope_properties(app: &mut MyApp, ui: &mut Ui, scope: &PortalResourceScope) {
    let resource_id = scope.arm_scope().expanded_form();
    ui.horizontal_wrapped(|ui| {
        ui.strong("Resource ID");
        ui.monospace(&resource_id);
        if ui.small_button("Copy ID").clicked() {
            ui.ctx().copy_text(resource_id);
        }
    });
    match scope {
        PortalResourceScope::ResourceGroup(id) => {
            if matches!(app.resource_groups, Loadable::NotLoaded) {
                load_resource_groups(app, false);
            }
            if let Some(group) = app
                .resource_groups
                .as_loaded()
                .and_then(|groups| groups.get(id))
                .cloned()
            {
                if ui.link(group.subscription_name.as_str()).clicked() {
                    app.navigate_tab(PortalTabTarget::Subscription(id.subscription_id));
                }
                ui.label(format!("Location · {}", group.location));
                show_properties(
                    app,
                    ui,
                    ReflectedValue::new(group.as_ref()),
                    &scope.overview(),
                );
            } else {
                unavailable(ui, &app.resource_groups, "Resource group");
            }
        }
        PortalResourceScope::Subscription(id) => {
            if matches!(app.subscriptions, Loadable::NotLoaded) {
                load_subscriptions(app);
            }
            if let Some(subscription) = app
                .subscriptions
                .as_loaded()
                .and_then(|items| items.iter().find(|item| item.id == *id))
                .cloned()
            {
                ui.label(format!("Tenant · {}", subscription.tenant_id));
                show_properties(
                    app,
                    ui,
                    ReflectedValue::new(subscription.as_ref()),
                    &scope.overview(),
                );
            } else {
                unavailable(ui, &app.subscriptions, "Subscription");
            }
        }
    }
}

fn show_properties(
    app: &mut MyApp,
    ui: &mut Ui,
    value: ReflectedValue<'_>,
    target: &PortalTabTarget,
) {
    ui.collapsing("Properties", |ui| {
        ScrollArea::vertical().max_height(210.0).show(ui, |ui| {
            if let Some(target) = value.show_links(ui, target) {
                app.navigate_tab(target);
            }
        });
    });
}

fn unavailable<T>(ui: &mut Ui, load: &Loadable<T>, label: &str) {
    match load {
        Loadable::NotLoaded | Loadable::Loading { .. } => {
            ui.spinner();
        }
        Loadable::Failed { error, .. } => {
            DiagnosticReportView::new(error).show(ui);
        }
        Loadable::Loaded { .. } => {
            ui.label(format!("{label} is unavailable for the selected account."));
        }
    }
}
