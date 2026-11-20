use crate::app::MyApp;
use crate::icons::page_icon;
use crate::portal_resource_scope::PortalResourceScope;
use crate::portal_scope_view::PortalScopeView;
use crate::portal_tab_target::PortalTabTarget;
use crate::reflected_value::ReflectedValue;
use crate::scope_activity::scope_activity_ui;
use crate::scope_iam::scope_iam_ui;
use crate::scope_navigation::scope_navigation;
use crate::windows::resources_window::scoped_resources_ui;
use crate::workers::load_resource_groups::load_resource_groups;
use crate::workers::load_subscriptions::load_subscriptions;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_app::OutputFormat;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_command::app_work::Loadable;
use eframe::egui::Align;
use eframe::egui::Button;
use eframe::egui::CentralPanel;
use eframe::egui::Frame;
use eframe::egui::Image;
use eframe::egui::Label;
use eframe::egui::Layout;
use eframe::egui::ScrollArea;
use eframe::egui::TextWrapMode;
use eframe::egui::Ui;
use eframe::egui::Vec2;
use teamy_tracing::gui::DiagnosticReportView;

pub(crate) fn scope_detail_ui(
    app: &mut MyApp,
    ui: &mut Ui,
    scope: &PortalResourceScope,
    view: Option<PortalScopeView>,
) {
    let overview = scope.overview();
    let name = app.portal_tab_title(&overview).into_owned();
    let mut inventory = false;
    let mut open_json = false;
    ui.horizontal(|ui| {
        let icon = Image::new(page_icon(scope.inventory())).fit_to_exact_size(Vec2::splat(26.0));
        inventory |= ui
            .add(Button::image(icon).frame(false))
            .on_hover_text(scope.inventory().title())
            .clicked();
        ui.vertical(|ui| {
            ui.heading(name);
            inventory |= ui
                .link(scope.label())
                .on_hover_text(scope.inventory().title())
                .clicked();
        });
        if view != Some(PortalScopeView::Json) {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                open_json = ui.button("View JSON").clicked();
            });
        }
    });
    ui.add_space(8.0);
    if inventory {
        app.navigate(scope.inventory());
        return;
    }
    if open_json {
        app.tabs.open_link_in_kept_tab(PortalTabTarget::ScopeView {
            scope: scope.clone(),
            view: PortalScopeView::Json,
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
            None | Some(PortalScopeView::Json) => {
                scope_properties(app, ui, scope, view == Some(PortalScopeView::Json));
                if view.is_none() {
                    ui.add_space(12.0);
                    scoped_resources_ui(app, ui, &scope.arm_scope());
                }
            }
        }
    });
}

fn scope_properties(app: &mut MyApp, ui: &mut Ui, scope: &PortalResourceScope, json: bool) {
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
                if json {
                    show_json(app, ui, CliOutput::facet(group));
                } else {
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
                }
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
                if json {
                    show_json(app, ui, CliOutput::facet(subscription));
                } else {
                    ui.label(format!("Tenant · {}", subscription.tenant_id));
                    show_properties(
                        app,
                        ui,
                        ReflectedValue::new(subscription.as_ref()),
                        &scope.overview(),
                    );
                }
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

fn show_json(app: &MyApp, ui: &mut Ui, output: CliOutput) {
    match output.render(Some(OutputFormat::Json), false, app.executor.cancellation()) {
        Ok(Some(text)) => {
            if ui.button("Copy JSON").clicked() {
                ui.ctx().copy_text(text.clone());
            }
            ScrollArea::both().show(ui, |ui| {
                ui.add(
                    Label::new(eframe::egui::RichText::new(text).monospace())
                        .selectable(true)
                        .wrap_mode(TextWrapMode::Extend),
                );
            });
        }
        Ok(None) => {
            ui.label("No JSON output.");
        }
        Err(error) => {
            ui.label(format!("Could not render JSON · {error}"));
        }
    }
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
