use crate::app::MyApp;
use crate::azure_account_action::AzureAccountAction;
use crate::azure_account_operation::AzureAccountOperation;
use crate::azure_account_operation_event::AzureAccountOperationEvent;
use crate::azure_account_picker::AzureAccountPicker;
use crate::azure_account_picker_action::AzureAccountPickerAction;
use cloud_terrastodon_auth::CloudTerrastodonAccountCatalogue;
use cloud_terrastodon_auth::CredentialAuthenticationProvider;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountNamespace;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountSession;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialRegistration;
use eframe::egui::Context;
use eframe::egui::Ui;
use std::sync::Arc;
use teamy_tracing::DiagnosticReport;

impl MyApp {
    pub(crate) fn handle_account_picker_action(
        &mut self,
        action: AzureAccountPickerAction,
        ctx: &Context,
    ) {
        match action {
            AzureAccountPickerAction::Select(selection) => self.select_account(selection),
            AzureAccountPickerAction::SignIn(selection) => {
                self.start_account_operation(
                    AzureAccountAction::sign_in(
                        selection.account_id,
                        AzureAccountPicker::selected_source(selection),
                    ),
                    ctx,
                );
            }
            AzureAccountPickerAction::Reauthenticate(selection) => {
                self.account_manager.open_reauthentication(
                    selection.account_id,
                    AzureAccountPicker::selected_source(selection),
                );
            }
            AzureAccountPickerAction::Manage(id) => self.account_manager.open_account(id),
            AzureAccountPickerAction::Add => self.account_manager.open_add(),
            AzureAccountPickerAction::Reload => {
                self.start_account_operation(AzureAccountAction::Reload, ctx)
            }
            AzureAccountPickerAction::ManageAccounts => self.account_manager.open(),
            AzureAccountPickerAction::ManageTenants => self.tenant_manager.open = true,
        }
    }

    fn select_account(&mut self, selection: AzureBearerAuthenticationSelection) {
        self.account_picker.selection_error = None;
        if self.authentication != Some(selection) {
            self.authentication = Some(selection);
            self.clear_account_inventory();
        }
    }

    pub(crate) fn ready_azure_authentication(&self) -> Option<AzureBearerAuthenticationSelection> {
        if self.account_operation.is_some() {
            return None;
        }
        let selection = self.authentication?;
        let registration =
            selected_registration(self.account_picker.catalogue.as_ref().ok()?, selection)?;
        matches!(
            registration.session,
            CloudTerrastodonAccountSession::Active { .. }
        )
        .then_some(selection)
    }

    pub(crate) fn show_azure_authentication_required(&mut self, ui: &mut Ui) -> bool {
        if self.ready_azure_authentication().is_some() {
            return false;
        }
        if self.account_operation.is_some() {
            ui.spinner();
        } else if let Some(selection) = self.authentication
            && self
                .account_picker
                .catalogue
                .as_ref()
                .ok()
                .and_then(|catalogue| selected_registration(catalogue, selection))
                .is_some()
        {
            if ui.button("Sign in").clicked() {
                self.handle_account_picker_action(
                    AzureAccountPickerAction::SignIn(selection),
                    ui.ctx(),
                );
            }
        } else if ui.button("Choose account").clicked() {
            self.account_picker.open();
            ui.ctx().request_repaint();
        }
        true
    }

    pub(crate) fn open_selected_account_sign_in(&mut self) {
        if let Some(selection) = self.ready_azure_authentication() {
            self.account_manager.open_reauthentication(
                selection.account_id,
                AzureAccountPicker::selected_source(selection),
            );
        }
    }

    fn start_account_operation(&mut self, action: AzureAccountAction, ctx: &Context) {
        if self.account_operation.is_some() {
            return;
        }
        match AzureAccountOperation::start(
            action,
            self.account_store.clone(),
            self.account_authentication.clone(),
            &self.executor,
            &self.work_tracker,
            ctx,
        ) {
            Ok(operation) => {
                self.account_picker.selection_error = None;
                self.account_operation = Some(operation);
                self.clear_account_inventory();
            }
            Err(error) => self.show_account_error(DiagnosticReport::new(error)),
        }
    }

    pub(crate) fn show_account_management(&mut self, ctx: &Context) {
        let busy = self.account_operation.is_some();
        if let Some(action) =
            self.account_manager
                .show(ctx, self.account_picker.catalogue.as_ref().ok(), busy)
        {
            self.start_account_operation(action, ctx);
        }
        if let Some(operation) = &self.account_operation {
            operation.show(ctx);
        }
        if self.account_operation.is_none() {
            let authentication = self.ready_azure_authentication();
            self.tenant_manager
                .show(ctx, &self.work_tracker, &self.executor, authentication);
        }
    }

    pub(crate) fn poll_account_operation(&mut self) {
        let Some(AzureAccountOperationEvent::Finished {
            outcome,
            catalogue,
            provider_closed,
        }) = self
            .account_operation
            .as_mut()
            .and_then(AzureAccountOperation::poll)
        else {
            return;
        };
        let login_target = self
            .account_operation
            .take()
            .and_then(|operation| operation.login_target);
        self.apply_account_catalogue(catalogue, provider_closed);
        match outcome {
            Ok(Some(new_account)) if self.account_manager.is_open() => {
                self.account_manager.open_account(new_account);
            }
            Ok(Some(_)) => {}
            Ok(None) => {
                if let Some((account_id, source)) = login_target
                    && let Ok(catalogue) = &self.account_picker.catalogue
                    && let Ok(account) = catalogue.by_id(account_id)
                    && let CloudTerrastodonAccountNamespace::Entra { tenant_id } = account.namespace
                    && let Some(auth_source) = source
                        .auth_source()
                        .and_then(|source| source.try_into().ok())
                {
                    self.select_account(AzureBearerAuthenticationSelection {
                        account_id,
                        auth_source,
                        tenant_id,
                    });
                }
            }
            Err(error) => self.show_account_error(error),
        }
    }

    fn show_account_error(&mut self, report: DiagnosticReport) {
        if self.account_manager.is_open() {
            self.account_manager.show_error(report);
        } else {
            self.account_picker.selection_error = Some(report);
            self.account_picker.open();
        }
    }

    fn apply_account_catalogue(
        &mut self,
        catalogue: Result<CloudTerrastodonAccountCatalogue, DiagnosticReport>,
        provider_closed: bool,
    ) {
        // Wait for the previous provider's leases before admitting any work
        // against the new catalogue. Workers hold only the provider they used.
        if provider_closed {
            let authentication = Arc::new(CredentialAuthenticationProvider::from_store(
                self.account_store.clone(),
            ));
            match self.executor.with_authentication(authentication.clone()) {
                Ok(executor) => {
                    self.executor = executor;
                    self.account_authentication = authentication;
                }
                Err(error) => {
                    self.account_picker.catalogue = Err(DiagnosticReport::new(error));
                    return;
                }
            }
        }
        if let Ok(next) = &catalogue
            && self
                .authentication
                .is_some_and(|old| selected_registration(next, old).is_none())
        {
            self.authentication = None;
        }
        self.account_picker.catalogue = catalogue;
    }
}

fn selected_registration(
    catalogue: &CloudTerrastodonAccountCatalogue,
    selection: AzureBearerAuthenticationSelection,
) -> Option<&CloudTerrastodonCredentialRegistration> {
    let account = catalogue.by_id(selection.account_id).ok()?;
    if account.namespace
        != (CloudTerrastodonAccountNamespace::Entra {
            tenant_id: selection.tenant_id,
        })
    {
        return None;
    }
    account
        .registration(AzureAccountPicker::selected_source(selection))
        .ok()
}
