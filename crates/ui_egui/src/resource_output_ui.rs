use crate::app::MyApp;
use crate::portal_resource_output_target::PortalResourceOutputTarget;
use crate::portal_resource_scope::PortalResourceScope;
use crate::portal_tab_target::PortalTabTarget;
use crate::resource_output::format_label;
use crate::resource_snapshot::ResourceSnapshot;
use cloud_terrastodon_app::OutputFormat;
use eframe::egui::ComboBox;
use eframe::egui::Frame;
use eframe::egui::Margin;
use eframe::egui::Ui;

/// Inspect a resource snapshot in any of the supported CLI output formats.
pub(crate) fn resource_output_ui(
    app: &mut MyApp,
    ui: &mut Ui,
    resource: &PortalResourceOutputTarget,
    format: OutputFormat,
) {
    Frame::NONE
        .inner_margin(Margin::symmetric(16, 8))
        .show(ui, |ui| {
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
            if app.show_azure_authentication_required(ui) {
                return;
            }

            let scope = match resource {
                PortalResourceOutputTarget::Scope(scope) => scope.clone(),
                PortalResourceOutputTarget::StorageAccount(id) => {
                    PortalResourceScope::StorageAccount(id.clone())
                }
            };
            ui.push_id(resource_id, |ui| {
                if let Some(snapshot) = ResourceSnapshot::load(app, ui, &scope) {
                    snapshot.show_output(app, ui, format);
                }
            });
        });
}
