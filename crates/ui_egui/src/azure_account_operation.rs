use crate::azure_account_action::AzureAccountAction;
use crate::azure_account_details::AzureAccountDetails;
use crate::azure_account_operation_event::AzureAccountOperationEvent;
use crate::azure_account_operation_origin::AzureAccountOperationOrigin;
use crate::azure_account_registration_form::source_label;
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
                let outcome = job.await.map_err(DiagnosticReport::new);
                publish(AzureAccountOperationEvent::RefreshingCatalogue);
                // A cancelled or failed operation may have completed a durable
                // lifecycle step. Read the catalogue even when it returns an error.
                let closed = authentication
                    .close_and_drain(executor.cancellation())
                    .await;
                let provider_closed = closed.is_ok();
                let catalogue = closed
                    .and_then(|()| store.load())
                    .map_err(DiagnosticReport::new);
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
                    let catalogue = Err(DiagnosticReport::new(eyre::eyre!(
                        "Account operation ended before returning a catalogue snapshot"
                    )));
                    return Some(AzureAccountOperationEvent::Finished {
                        outcome: Err(DiagnosticReport::new(eyre::eyre!(
                            "Account operation ended before reporting an outcome. Reload to check the account status."
                        ))),
                        catalogue,
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
}
