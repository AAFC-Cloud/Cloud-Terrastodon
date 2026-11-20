use crate::app::MyApp;
use crate::portal_resource_output_target::PortalResourceOutputTarget;
use crate::portal_resource_scope::PortalResourceScope;
use crate::portal_tab_target::PortalTabTarget;
use crate::resource_output::format_label;
use crate::storage_account_browser::load_storage_account;
use crate::workers::load_resource_groups::load_resource_groups;
use crate::workers::load_subscriptions::load_subscriptions;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_app::OutputFormat;
use cloud_terrastodon_azure_cli::storage::account::AzureStorageAccountShowArgs;
use cloud_terrastodon_command::app_work::Loadable;
use eframe::egui::ComboBox;
use eframe::egui::Ui;
use teamy_tracing::gui::DiagnosticReportView;

/// Inspect a resource snapshot in any of the supported CLI output formats.
pub(crate) fn resource_output_ui(
    app: &mut MyApp,
    ui: &mut Ui,
    resource: &PortalResourceOutputTarget,
    format: OutputFormat,
) {
    if app.show_azure_authentication_required(ui) {
        return;
    }
    let resource_id = resource.resource_id();
    ui.horizontal_wrapped(|ui| {
        ui.strong("Resource ID");
        ui.monospace(&resource_id);
        if ui.small_button("Copy ID").clicked() {
            ui.ctx().copy_text(resource_id.clone());
        }
    });
    let mut selected_format = format;
    ui.horizontal(|ui| {
        ui.label("Format");
        ComboBox::from_id_salt((&resource_id, "resource-output-format"))
            .selected_text(format_label(format))
            .show_ui(ui, |ui| {
                for format in [
                    OutputFormat::Json,
                    OutputFormat::FacetPretty,
                    OutputFormat::Text,
                ] {
                    ui.selectable_value(&mut selected_format, format, format_label(format));
                }
            });
    });
    if selected_format != format {
        app.navigate_current_tab(PortalTabTarget::ResourceOutput {
            resource: resource.clone(),
            format: selected_format,
        });
        return;
    }

    ui.push_id(resource_id, |ui| match resource {
        PortalResourceOutputTarget::Scope(PortalResourceScope::ResourceGroup(id)) => {
            if matches!(app.resource_groups, Loadable::NotLoaded) {
                load_resource_groups(app, false);
            }
            if let Some(group) = app
                .resource_groups
                .as_loaded()
                .and_then(|groups| groups.get(id))
            {
                app.resource_output.show(
                    ui,
                    format,
                    group,
                    CliOutput::facet,
                    app.executor.cancellation(),
                );
            } else if unavailable(ui, &app.resource_groups, "Resource group") {
                load_resource_groups(app, false);
            }
        }
        PortalResourceOutputTarget::Scope(PortalResourceScope::Subscription(id)) => {
            if matches!(app.subscriptions, Loadable::NotLoaded) {
                load_subscriptions(app);
            }
            if let Some(subscription) = app
                .subscriptions
                .as_loaded()
                .and_then(|subscriptions| subscriptions.iter().find(|item| item.id == *id))
            {
                app.resource_output.show(
                    ui,
                    format,
                    subscription,
                    CliOutput::facet,
                    app.executor.cancellation(),
                );
            } else if unavailable(ui, &app.subscriptions, "Subscription") {
                load_subscriptions(app);
            }
        }
        PortalResourceOutputTarget::StorageAccount(id) => {
            if !app.storage_account_browser.entries.contains_key(id) {
                load_storage_account(app, id, false);
            }
            let Some(inspection) = app.storage_account_browser.entries.get(id) else {
                return;
            };
            if let Some(account) = inspection.response.as_loaded() {
                app.resource_output.show(
                    ui,
                    format,
                    account,
                    AzureStorageAccountShowArgs::output,
                    app.executor.cancellation(),
                );
            } else if unavailable(ui, &inspection.response, "Storage account") {
                load_storage_account(app, id, false);
            }
        }
    });
}

fn unavailable<T>(ui: &mut Ui, load: &Loadable<T>, label: &str) -> bool {
    match load {
        Loadable::NotLoaded | Loadable::Loading { .. } => {
            ui.spinner();
        }
        Loadable::Failed { error, .. } => {
            ui.label(format!("{label} could not be loaded."));
            ui.collapsing("Error details", |ui| {
                DiagnosticReportView::new(error).show(ui);
            });
            return ui.button("Retry").clicked();
        }
        Loadable::Loaded { .. } => {
            ui.label(format!("{label} is unavailable for the selected account."));
        }
    }
    false
}
