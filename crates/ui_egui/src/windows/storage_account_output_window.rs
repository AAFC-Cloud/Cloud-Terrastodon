use crate::app::MyApp;
use crate::portal_tab_target::PortalTabTarget;
use crate::storage_account_browser::load_storage_account;
use crate::storage_account_output::format_label;
use cloud_terrastodon_app::OutputFormat;
use cloud_terrastodon_azure::AzureStorageAccountId;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_command::app_work::Loadable;
use eframe::egui::ComboBox;
use eframe::egui::Ui;
use teamy_tracing::gui::DiagnosticReportView;

/// Inspect the same account response as the overview using the CLI renderers.
pub(crate) fn storage_account_output_ui(
    app: &mut MyApp,
    ui: &mut Ui,
    id: &AzureStorageAccountId,
    format: OutputFormat,
) {
    if app.show_azure_authentication_required(ui) {
        return;
    }
    ui.horizontal_wrapped(|ui| {
        ui.strong("Resource ID");
        let resource_id = id.expanded_form();
        ui.monospace(&resource_id);
        if ui.small_button("Copy ID").clicked() {
            ui.ctx().copy_text(resource_id);
        }
    });
    let mut selected_format = format;
    ui.horizontal(|ui| {
        ui.label("Format");
        ComboBox::from_id_salt((id, "storage-account-output-format"))
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
        app.navigate_current_tab(PortalTabTarget::StorageAccountOutput {
            id: id.clone(),
            format: selected_format,
        });
        return;
    }

    if !app.storage_account_browser.entries.contains_key(id) {
        load_storage_account(app, id, false);
    }
    let Some(inspection) = app.storage_account_browser.entries.get_mut(id) else {
        return;
    };
    match &inspection.response {
        Loadable::Loaded { value, .. } => {
            ui.push_id(id, |ui| {
                inspection
                    .output
                    .show(ui, format, value, app.executor.cancellation());
            });
        }
        Loadable::Failed { error, .. } => {
            ui.label("Could not load the storage account");
            ui.collapsing("Error details", |ui| {
                DiagnosticReportView::new(error).show(ui)
            });
            if ui.button("Retry").clicked() {
                load_storage_account(app, id, false);
            }
        }
        Loadable::Loading { .. } | Loadable::NotLoaded => {
            ui.spinner();
        }
    }
}
