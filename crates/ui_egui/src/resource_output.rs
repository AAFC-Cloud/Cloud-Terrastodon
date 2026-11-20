use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_app::OutputFormat;
use eframe::egui::Label;
use eframe::egui::RichText;
use eframe::egui::ScrollArea;
use eframe::egui::TextWrapMode;
use eframe::egui::Ui;
use eyre::Result;
use std::any::Any;
use std::sync::Arc;
use std::sync::Weak;
use teamy_cancellation::CancellationScope;

/// Last rendered representation of a resource snapshot.
#[derive(Default)]
pub(crate) struct ResourceOutput {
    rendered: Option<(OutputFormat, Weak<dyn Any>, Result<Option<String>>)>,
}

impl ResourceOutput {
    pub(crate) fn show<T: 'static>(
        &mut self,
        ui: &mut Ui,
        format: OutputFormat,
        value: &Arc<T>,
        output: impl FnOnce(Arc<T>) -> CliOutput,
        cancellation: &CancellationScope,
    ) {
        let identity: Weak<dyn Any> = Arc::<T>::downgrade(value);
        if self
            .rendered
            .as_ref()
            .is_none_or(|(rendered_format, current, _)| {
                *rendered_format != format || !Weak::ptr_eq(current, &identity)
            })
        {
            let rendered = output(Arc::clone(value))
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
            self.rendered = Some((format, identity, rendered));
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

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Context;
    use std::cell::Cell;
    use teamy_cancellation::CancellationToken;

    #[test]
    fn preview_renders_again_when_format_or_snapshot_changes() -> Result<()> {
        CancellationToken::new().with_scope(|cancellation| {
            let context = Context::default();
            let mut preview = ResourceOutput::default();
            let calls = Cell::new(0);
            let first = Arc::new(7_u32);
            let replacement = Arc::new(7_u32);
            let mut frame = |format, value: &Arc<u32>| {
                let mut frame = context.run_ui(Default::default(), |ui| {
                    preview.show(
                        ui,
                        format,
                        value,
                        |value| {
                            calls.set(calls.get() + 1);
                            CliOutput::facet(value)
                        },
                        &cancellation,
                    );
                });
                // Headless tests have no renderer to apply font texture uploads.
                frame.textures_delta.clear();
            };

            frame(OutputFormat::Json, &first);
            frame(OutputFormat::Json, &first);
            assert_eq!(calls.get(), 1);
            frame(OutputFormat::Text, &first);
            assert_eq!(calls.get(), 2);
            frame(OutputFormat::Text, &replacement);
            assert_eq!(calls.get(), 3);
            assert_eq!(Arc::strong_count(&replacement), 1);
            Ok(())
        })
    }
}
