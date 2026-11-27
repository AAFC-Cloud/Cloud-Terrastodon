use crate::azure_account_action::AzureAccountAction;
use crate::azure_account_details::AzureAccountDetails;
use crate::azure_account_operation_event::AzureAccountOperationEvent;
use crate::azure_account_operation_origin::AzureAccountOperationOrigin;
use crate::azure_account_registration_form::source_label;
use crate::capture_gui_failure::capture_gui_failure;
use crate::gui_work_cache::GuiWorkCache;
use cloud_terrastodon_auth::AccountSignInPrompt;
use cloud_terrastodon_auth::CloudTerrastodonAccountCatalogue;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_auth::CredentialAuthenticationProvider;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialSource;
use cloud_terrastodon_command::app_work::AppWorkTracker;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eframe::egui::Context;
use eframe::egui::Id;
use eframe::egui::Window;
use eyre::Result;
use std::sync::Arc;
use teamy_cancellation::CancellationToken;
use teamy_tracing::DiagnosticReport;
use teamy_tracing::gui::DiagnosticReportView;
use tokio::runtime::Handle;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::error::TryRecvError;

/// Owns one local account operation and its private sign-in prompts.
pub(crate) struct AzureAccountOperation {
    pub origin: AzureAccountOperationOrigin,
    pub login_target: Option<(CloudTerrastodonAccountId, CloudTerrastodonCredentialSource)>,
    events: UnboundedReceiver<AzureAccountOperationEvent>,
    cancellation: Option<CancellationToken>,
    prompt: Option<AccountSignInPrompt>,
    refreshing_catalogue: bool,
}

impl AzureAccountOperation {
    #[allow(
        clippy::too_many_arguments,
        reason = "explicit account operation dependencies"
    )]
    pub fn start(
        action: AzureAccountAction,
        origin: AzureAccountOperationOrigin,
        store: CloudTerrastodonAccountStore,
        authentication: Arc<CredentialAuthenticationProvider>,
        executor: &CloudTerrastodonExecutor,
        work_cache: &GuiWorkCache,
        work_tracker: &AppWorkTracker,
        ctx: &Context,
    ) -> Result<Self> {
        let login_target = action.login_target();
        let (tx, events) = mpsc::unbounded_channel();
        let job_executor = executor.clone();
        let work_cache = work_cache.clone();
        let ctx = ctx.clone();
        work_tracker.spawn(
            executor.cancellation(),
            &Handle::current(),
            "manage Cloud Terrastodon account".to_owned(),
            async move |scope| {
                let publish = |event| {
                    // Closing the GUI naturally drops the receiver.
                    let _ = tx.send(event);
                    ctx.request_repaint();
                };
                let executor = job_executor.with_cancellation_scope(scope);
                let (cancel, job) = executor.with_async_child_scope(async |executor| {
                    action
                        .run(&store, &executor, &work_cache, &|prompt| {
                            publish(AzureAccountOperationEvent::Prompt(prompt));
                        })
                        .await
                });
                publish(AzureAccountOperationEvent::Started(cancel));
                let outcome = job
                    .await
                    .map_err(|error| capture_gui_failure(error, "Account operation failed"));
                publish(AzureAccountOperationEvent::RefreshingCatalogue);
                // A cancelled or failed operation may have completed a durable
                // lifecycle step. Read the catalogue even when it returns an error.
                let closed = authentication
                    .close_and_drain(executor.cancellation())
                    .await;
                let provider_closed = closed.is_ok();
                let catalogue = closed.and_then(|()| store.load()).map_err(|error| {
                    capture_gui_failure(error, "Could not reload account catalogue")
                });
                publish(AzureAccountOperationEvent::Finished {
                    outcome,
                    catalogue,
                    provider_closed,
                });
                eyre::Ok(())
            },
        )?;
        Ok(Self {
            origin,
            login_target,
            events,
            cancellation: None,
            prompt: None,
            refreshing_catalogue: false,
        })
    }

    pub fn poll(&mut self) -> Option<AzureAccountOperationEvent> {
        loop {
            match self.events.try_recv() {
                Ok(AzureAccountOperationEvent::Started(token)) => self.cancellation = Some(token),
                Ok(AzureAccountOperationEvent::Prompt(prompt)) => self.prompt = Some(prompt),
                Ok(AzureAccountOperationEvent::RefreshingCatalogue) => {
                    self.prompt = None;
                    self.cancellation = None;
                    self.refreshing_catalogue = true;
                }
                Ok(finished @ AzureAccountOperationEvent::Finished { .. }) => {
                    return Some(finished);
                }
                Err(TryRecvError::Empty) => return None,
                Err(TryRecvError::Disconnected) => {
                    let error = capture_gui_failure(
                        eyre::eyre!(
                            "Account operation ended before reporting an outcome. Reload to check the account status."
                        ),
                        "Account operation was interrupted",
                    );
                    return Some(AzureAccountOperationEvent::Finished {
                        outcome: Err(error.clone()),
                        catalogue: Err(error),
                        provider_closed: false,
                    });
                }
            }
        }
    }

    pub fn show(&self, ctx: &Context, catalogue: Option<&CloudTerrastodonAccountCatalogue>) {
        Window::new(if self.login_target.is_some() {
            "Account sign-in"
        } else {
            "Updating account"
        })
        .id(Id::new("account_operation"))
        .collapsible(false)
        .show(ctx, |ui| {
            if let Some((account_id, source)) = self.login_target
                && let Some(account) = catalogue.and_then(|value| value.by_id(account_id).ok())
            {
                ui.strong(account.alias.as_str());
                ui.label(AzureAccountDetails::namespace_label(&account.namespace));
                ui.label(source_label(source));
            }
            let cancelling = self
                .cancellation
                .as_ref()
                .is_some_and(CancellationToken::is_cancelled);
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(if self.refreshing_catalogue {
                    "Reloading accounts…"
                } else if cancelling {
                    "Cancelling…"
                } else if self.login_target.is_some() {
                    "Waiting for sign-in…"
                } else {
                    "Saving account changes…"
                });
            });
            match self.prompt.as_ref().filter(|_| !cancelling) {
                Some(AccountSignInPrompt::AzureCliDeviceCode { code }) => {
                    ui.label("Enter this code in your browser.");
                    ui.horizontal(|ui| {
                        ui.monospace(code);
                        if ui.button("Copy code").clicked() {
                            ui.ctx().copy_text(code.clone());
                        }
                    });
                    ui.hyperlink_to(
                        "Open Microsoft device sign-in",
                        AccountSignInPrompt::AZURE_CLI_DEVICE_SIGN_IN_URL,
                    );
                }
                Some(AccountSignInPrompt::BrowserAuthorization { url }) => {
                    ui.hyperlink_to("Open browser sign-in", url);
                }
                None => {}
            }
            if !cancelling
                && let Some(cancellation) = &self.cancellation
                && ui.button("Cancel").clicked()
            {
                cancellation.request_cancel("account operation cancelled in the GUI");
            }
        });
    }

    /// Show the result in place of the sign-in progress until it is dismissed.
    pub fn show_sign_in_failure(ctx: &Context, error: &DiagnosticReport) -> bool {
        let mut open = true;
        let dismissed = Window::new("Sign-in failed")
            .id(Id::new("account_operation"))
            .default_width(430.0)
            .collapsible(false)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label(error.report.to_string());
                ui.collapsing("Error details", |ui| {
                    DiagnosticReportView::new(error).show(ui);
                });
                ui.button("Close").clicked()
            })
            .and_then(|window| window.inner)
            .unwrap_or(false);
        dismissed || !open
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Event;
    use eframe::egui::Modifiers;
    use eframe::egui::PointerButton;
    use eframe::egui::Pos2;
    use eframe::egui::RawInput;
    use eframe::egui::Rect;
    use eframe::egui::Shape;
    use eframe::egui::Vec2;

    #[test]
    fn failed_sign_in_shows_the_reason_until_dismissed() {
        let reason = "credential authenticates a different principal; register a separate account";
        let report = DiagnosticReport::new(eyre::eyre!(reason));
        let context = Context::default();
        let draw = |events| {
            let mut dismissed = false;
            let mut output = context.run_ui(
                RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(700.0, 480.0))),
                    events,
                    ..Default::default()
                },
                |ui| dismissed |= AzureAccountOperation::show_sign_in_failure(ui.ctx(), &report),
            );
            output.textures_delta.clear();
            (output.shapes, dismissed)
        };
        draw(Vec::new());
        let (shapes, dismissed) = draw(Vec::new());
        assert!(!dismissed);
        let text_position = |label| {
            shapes.iter().find_map(|clipped| match &clipped.shape {
                Shape::Text(text) if text.galley.text() == label => {
                    Some(text.visual_bounding_rect().center())
                }
                _ => None,
            })
        };
        assert!(text_position(reason).is_some());
        let pos = text_position("Close").unwrap();
        let click = |pressed| {
            vec![
                Event::PointerMoved(pos),
                Event::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                },
            ]
        };
        assert!(!draw(click(true)).1);
        assert!(draw(click(false)).1);
    }
}
