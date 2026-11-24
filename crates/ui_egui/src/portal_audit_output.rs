use crate::app::MyApp;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_app::OutputFormat;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eframe::egui::Button;
use eframe::egui::Checkbox;
use eframe::egui::Id;
use eframe::egui::Label;
use eframe::egui::RichText;
use eframe::egui::ScrollArea;
use eframe::egui::TextWrapMode;
use eframe::egui::Ui;
use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;
use std::time::Instant;
use teamy_tracing::gui::DiagnosticReportView;
use teamy_tracing::gui::escape_display;

/// Display an audit that supplies CLI text output.
pub(crate) fn show_audit_output<F>(
    app: &mut MyApp,
    ui: &mut Ui,
    name: &'static str,
    audit: impl FnOnce(CloudTerrastodonExecutor, bool) -> F + Send + 'static,
) where
    F: Future<Output = eyre::Result<CliOutput>> + Send + 'static,
{
    show_audit_result(
        app,
        ui,
        name,
        async move |executor, clean| {
            let cancellation = executor.cancellation().clone();
            let output = audit(executor, clean).await?;
            let text = output
                .render(Some(OutputFormat::Text), false, &cancellation)?
                .unwrap_or_default();
            let text = if text.contains('\x1b') {
                strip_ansi_escapes::strip_str(text)
            } else {
                text
            };
            Ok(escape_display(&text))
        },
        |_, ui, value: &String, result_id| {
            if value.is_empty() {
                ui.label("Audit completed.");
                return;
            }
            if ui.button("Copy report").clicked() {
                ui.ctx().copy_text(value.clone());
            }
            ScrollArea::both().id_salt(result_id).show(ui, |ui| {
                ui.add(
                    Label::new(RichText::new(value).monospace())
                        .selectable(true)
                        .wrap_mode(TextWrapMode::Extend),
                );
            });
        },
    );
}

/// Run an audit and retain its completed result for the view to render.
pub(crate) fn show_audit_result<T, F>(
    app: &mut MyApp,
    ui: &mut Ui,
    name: &'static str,
    audit: impl FnOnce(CloudTerrastodonExecutor, bool) -> F + Send + 'static,
    show_result: impl FnOnce(&mut MyApp, &mut Ui, &T, Id),
) where
    T: Debug + Send + Sync + 'static,
    F: Future<Output = eyre::Result<T>> + Send + 'static,
{
    let account_generation = app.azure.account_generation;
    let result_id = ui.make_persistent_id(("audit-result", name));
    let clean_id = result_id.with("clean");
    let result = app
        .azure
        .audit_results
        .get(&result_id)
        .and_then(|result| Arc::clone(result).downcast::<Loadable<T>>().ok());
    let loading = matches!(result.as_deref(), Some(Loadable::Loading { .. }));
    let mut clean = ui.data(|data| data.get_temp::<bool>(clean_id).unwrap_or(false));
    let submit = ui
        .horizontal_wrapped(|ui| {
            let submit = ui.add_enabled(!loading, Button::new("Run audit")).clicked();
            if ui
                .add_enabled(!loading, Checkbox::new(&mut clean, "Refresh cached data"))
                .on_hover_text("Refresh audit inventories before fetching results (--clean)")
                .changed()
            {
                ui.data_mut(|data| data.insert_temp(clean_id, clean));
            }
            submit
        })
        .inner;

    if submit {
        let executor = app.executor.clone();
        let cancellation = executor.cancellation().clone();
        let work_cache = app.work_cache.clone();
        let context = ui.ctx().clone();
        let started_at = Instant::now();
        let queued = LoadableWorkBuilder::<MyApp, T>::new()
            .description(name)
            .setter(move |app, result| {
                if let Some(azure) = app.azure_content_mut(account_generation) {
                    azure.audit_results.insert(result_id, Arc::new(result));
                    context.request_repaint();
                }
            })
            .work(async move |scope| {
                let executor =
                    work_cache.job_executor(&executor.with_cancellation_scope(scope.clone()))?;
                audit(executor, clean).await
            })
            .build()
            .enqueue(
                &app.work_tracker.clone(),
                app.tx.clone(),
                app,
                &cancellation,
            );
        if let Err(error) = queued {
            let result = Loadable::<T>::Failed {
                error: error.into(),
                started_at,
                finished_at: Instant::now(),
            };
            app.azure.audit_results.insert(result_id, Arc::new(result));
            ui.ctx().request_repaint();
        }
        return;
    }

    match result.as_deref() {
        Some(Loadable::Loading { .. }) => {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Running audit…");
            });
        }
        Some(Loadable::Loaded { value, .. }) => {
            if app.egui_config.demo_mode {
                ui.label("Audit completed. Report details are hidden in demo mode.");
            } else {
                show_result(app, ui, value, result_id);
            }
        }
        Some(Loadable::Failed { error, .. }) => {
            ui.label("Could not complete the audit.");
            if !app.egui_config.demo_mode {
                ui.collapsing("Error details", |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        DiagnosticReportView::new(error).show(ui);
                    });
                });
            }
        }
        Some(Loadable::NotLoaded) | None => {}
    }
}
