use crate::app::MyApp;
use crate::portal_audit_output::show_audit_output;
use crate::portal_audit_output::show_audit_result;
use crate::portal_section_tabs::portal_section_tabs;
use crate::portal_storage_audit_findings::show_storage_audit_findings;
use crate::portal_storage_audit_format::PortalStorageAuditFormat;
use crate::resource_output::ResourceOutput;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_azure_cli::AzureAuditReport;
use cloud_terrastodon_azure_cli::AzureStorageAccountAuditReport;
use cloud_terrastodon_azure_cli::AzureStorageAccountAuditRequest;
use cloud_terrastodon_azure_cli::audit_azure;
use cloud_terrastodon_execution::ExecutableRequest;
use eframe::egui::Ui;
use std::sync::Arc;

pub(crate) fn azure_audit_ui(app: &mut MyApp, ui: &mut Ui) {
    let Some(authentication) = app.ready_azure_authentication() else {
        app.show_azure_authentication_required(ui);
        return;
    };
    show_audit_output(app, ui, "Azure audit", async move |executor, clean| {
        let report = audit_azure(authentication, clean, &executor).await?;
        Ok(CliOutput::facet_with_text(
            report,
            AzureAuditReport::write_text,
        ))
    });
}

pub(crate) fn storage_account_audit_ui(app: &mut MyApp, ui: &mut Ui) {
    let Some(authentication) = app.ready_azure_authentication() else {
        app.show_azure_authentication_required(ui);
        return;
    };
    show_audit_result(
        app,
        ui,
        "Storage account audit",
        async move |executor, clean| {
            let report = AzureStorageAccountAuditRequest { authentication }
                .execute(&executor)
                .with_invalidation(clean)
                .await?;
            eyre::Ok(Arc::new(report))
        },
        |app, ui, report: &Arc<AzureStorageAccountAuditReport>, result_id| {
            let format_id = result_id.with("format");
            let mut format = ui
                .data(|data| data.get_temp::<PortalStorageAuditFormat>(format_id))
                .unwrap_or_default();
            let previous_format = format;
            ui.horizontal_wrapped(|ui| {
                portal_section_tabs(
                    ui,
                    &mut format,
                    PortalStorageAuditFormat::CHOICES,
                    PortalStorageAuditFormat::label,
                );
            });
            if format != previous_format {
                ui.data_mut(|data| data.insert_temp(format_id, format));
            }
            match format {
                PortalStorageAuditFormat::Findings => {
                    show_storage_audit_findings(app, ui, report, result_id);
                }
                PortalStorageAuditFormat::Cli(format) => {
                    let output_id = result_id.with("rendered");
                    let mut output = ui
                        .data_mut(|data| data.remove_temp::<ResourceOutput>(output_id))
                        .unwrap_or_default();
                    output.show(
                        ui,
                        format,
                        report,
                        |report| {
                            CliOutput::facet_with_text(report, |report, writer, cancellation| {
                                report.write_text(writer, cancellation)
                            })
                        },
                        app.executor.cancellation(),
                    );
                    ui.data_mut(|data| data.insert_temp(output_id, output));
                }
            }
        },
    );
}
