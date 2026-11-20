use cloud_terrastodon_app::OutputFormat;
use cloud_terrastodon_azure::AzureStorageAccount;
use cloud_terrastodon_azure_cli::storage::account::AzureStorageAccountShowArgs;
use eframe::egui::Label;
use eframe::egui::RichText;
use eframe::egui::ScrollArea;
use eframe::egui::TextWrapMode;
use eframe::egui::Ui;
use eyre::Result;
use std::sync::Arc;
use teamy_cancellation::CancellationScope;

/// Last rendered representation of a storage account inspection.
#[derive(Default)]
pub(crate) struct StorageAccountOutput {
    rendered: Option<(
        OutputFormat,
        Arc<AzureStorageAccount>,
        Result<Option<String>>,
    )>,
}

impl StorageAccountOutput {
    pub(crate) fn show(
        &mut self,
        ui: &mut Ui,
        format: OutputFormat,
        value: &Arc<AzureStorageAccount>,
        cancellation: &CancellationScope,
    ) {
        if self
            .rendered
            .as_ref()
            .is_none_or(|(rendered_format, current, _)| {
                *rendered_format != format || !Arc::ptr_eq(current, value)
            })
        {
            let output = AzureStorageAccountShowArgs::output(value.clone());
            let rendered = output
                .render(Some(format), false, cancellation)
                .map(|preview| {
                    preview.map(|text| {
                        if format != OutputFormat::Json && text.contains('\x1b') {
                            strip_ansi_escapes::strip_str(text)
                        } else {
                            text
                        }
                    })
                });
            self.rendered = Some((format, value.clone(), rendered));
        }
        match &self.rendered {
            Some((_, _, Ok(Some(text)))) => {
                if ui.button("Copy output").clicked() {
                    ui.ctx().copy_text(text.clone());
                }
                ScrollArea::both()
                    .id_salt(format_label(format))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.add(
                            Label::new(RichText::new(text).monospace())
                                .selectable(true)
                                .wrap_mode(TextWrapMode::Extend),
                        );
                    });
            }
            Some((_, _, Ok(None))) => {
                ui.label("No output");
            }
            Some((_, _, Err(error))) => {
                ui.label(format!("Could not render the output · {error}"));
            }
            None => {}
        }
    }
}

pub(crate) fn format_label(format: OutputFormat) -> &'static str {
    match format {
        OutputFormat::Json => "JSON",
        OutputFormat::FacetPretty => "Facet Pretty",
        OutputFormat::Text => "Text",
        OutputFormat::Auto => "Auto",
    }
}
