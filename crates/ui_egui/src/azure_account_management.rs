use crate::app::MyApp;
use crate::azure_account_action::AzureAccountAction;
use crate::azure_account_operation::AzureAccountOperation;
use crate::azure_account_operation_event::AzureAccountOperationEvent;
use crate::azure_account_operation_origin::AzureAccountOperationOrigin;
use crate::azure_account_picker::AzureAccountPicker;
use crate::azure_account_picker_action::AzureAccountPickerAction;
use crate::azure_account_sign_in_method::AzureAccountSignInMethod;
use crate::capture_gui_failure::capture_gui_failure;
use crate::portal_page::PortalPage;
use crate::portal_tab_id::PortalTabId;
use crate::portal_tab_target::PortalTabTarget;
use cloud_terrastodon_auth::CloudTerrastodonAccountCatalogue;
use cloud_terrastodon_auth::CredentialAuthenticationProvider;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountNamespace;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountSession;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialRegistration;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialSource;
use cloud_terrastodon_azure::AzureTenantId;
use eframe::egui::CollapsingHeader;
use eframe::egui::Context;
use eframe::egui::Ui;
use std::sync::Arc;
use teamy_cancellation::CancellationError;
use teamy_tracing::DiagnosticReport;
use teamy_tracing::gui::DiagnosticReportView;

impl MyApp {
    pub(crate) fn handle_account_picker_action(
        &mut self,
        action: AzureAccountPickerAction,
        ctx: &Context,
    ) {
        match action {
            AzureAccountPickerAction::Select(selection) => {
                self.select_account(self.tabs.active_id(), selection);
            }
            AzureAccountPickerAction::SignIn(selection) => {
                self.start_account_operation(
                    AzureAccountAction::sign_in(
                        selection.account_id,
                        AzureAccountPicker::selected_source(selection),
                    ),
                    Some(selection),
                    ctx,
                );
            }
            AzureAccountPickerAction::SignInWithDeviceCode(account_id) => {
                let selection = self
                    .account_picker
                    .catalogue
                    .as_ref()
                    .ok()
                    .and_then(|catalogue| catalogue.by_id(account_id).ok())
                    .and_then(|account| match account.namespace {
                        CloudTerrastodonAccountNamespace::Entra { tenant_id } => {
                            Some(AzureBearerAuthenticationSelection {
                                account_id,
                                tenant_id,
                                auth_source: AzureBearerAuthSource::AzureCli,
                            })
                        }
                        _ => None,
                    });
                self.start_account_operation(
                    AzureAccountAction::Login {
                        account_id,
                        method: AzureAccountSignInMethod::AzureCliDeviceCode,
                    },
                    selection,
                    ctx,
                );
            }
            AzureAccountPickerAction::Reload => {
                self.start_account_operation(AzureAccountAction::Reload, None, ctx)
            }
            AzureAccountPickerAction::ManageAccounts => {
                self.tabs
                    .open_kept(PortalTabTarget::Page(PortalPage::Accounts));
                self.sync_azure_content();
            }
        }
    }

    fn select_account(&mut self, tab: PortalTabId, selection: AzureBearerAuthenticationSelection) {
        let Some(content) = self.azure_content_for_tab_mut(tab) else {
            return;
        };
        let changed = content.authentication != Some(selection);
        content.authentication = Some(selection);
        self.account_picker.set_selection_error(tab, None);
        self.egui_config.azure_authentication = Some(selection);
        if changed {
            self.reset_azure_content_for_tab(tab);
        }
    }

    pub(crate) fn ready_azure_authentication(&self) -> Option<AzureBearerAuthenticationSelection> {
        self.ready_azure_authentication_for(self.azure.authentication)
    }

    pub(crate) fn ready_azure_authentication_for(
        &self,
        selection: Option<AzureBearerAuthenticationSelection>,
    ) -> Option<AzureBearerAuthenticationSelection> {
        if self.account_operation.is_some() {
            return None;
        }
        let selection = selection?;
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
        } else if let Some(selection) = self.azure.authentication
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
            self.account_picker.open(self.tabs.active_id());
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
            self.tabs
                .open_kept(PortalTabTarget::Page(PortalPage::Accounts));
            self.sync_azure_content();
        }
    }

    fn start_account_operation(
        &mut self,
        action: AzureAccountAction,
        picker_selection: Option<AzureBearerAuthenticationSelection>,
        ctx: &Context,
    ) {
        if self.account_operation.is_some() {
            return;
        }
        let signing_in = action.login_target().is_some();
        if signing_in {
            self.account_sign_in_failure = None;
        }
        let tab = self.tabs.active_id();
        let origin = AzureAccountOperationOrigin {
            tab,
            generation: self.azure.account_generation,
            picker_selection,
        };
        match AzureAccountOperation::start(
            action,
            origin,
            self.account_store.clone(),
            self.account_authentication.clone(),
            &self.executor,
            &self.work_cache,
            &self.work_tracker,
            ctx,
        ) {
            Ok(operation) => {
                self.account_picker.set_selection_error(tab, None);
                self.account_operation = Some(operation);
            }
            Err(error) => self.show_account_error(
                tab,
                signing_in,
                capture_gui_failure(error, "Could not start account operation"),
            ),
        }
    }

    pub(crate) fn show_accounts_page(&mut self, ui: &mut Ui) {
        let busy = self.account_operation.is_some();
        self.tenant_manager.load_catalogue_if_needed(
            &self.work_tracker,
            &self.executor,
            &self.work_cache,
            ui.ctx(),
        );
        let mut open_tenants = false;
        if let Some(action) = self.account_manager.show(
            ui,
            self.account_picker.catalogue.as_ref().ok(),
            busy,
            self.tenant_manager.tracked_tenants(),
            &mut open_tenants,
        ) {
            self.start_account_operation(action, None, ui.ctx());
        }
        if open_tenants {
            self.tabs
                .open_kept(PortalTabTarget::Page(PortalPage::AzureTenants));
            self.sync_azure_content();
        }
    }

    pub(crate) fn show_account_operation(&mut self, ctx: &Context) {
        if let Some(operation) = &self.account_operation {
            operation.show(ctx, self.account_picker.catalogue.as_ref().ok());
        } else if let Some(error) = &self.account_sign_in_failure
            && AzureAccountOperation::show_sign_in_failure(ctx, error)
        {
            self.account_sign_in_failure = None;
        }
    }

    pub(crate) fn show_azure_tenants_page(&mut self, ui: &mut Ui, tenant: Option<AzureTenantId>) {
        if self.account_operation.is_some() {
            ui.spinner();
            return;
        }
        if let Some(error) = self.account_picker.selection_error(self.tabs.active_id()) {
            CollapsingHeader::new("Account operation failed")
                .id_salt(("tenant-account-operation-error", tenant))
                .show(ui, |ui| DiagnosticReportView::new(error).show(ui));
        }
        let accounts = match &self.account_picker.catalogue {
            Ok(catalogue) => Some(catalogue),
            Err(error) => {
                CollapsingHeader::new("Could not read registered accounts")
                    .id_salt(("tenant-account-catalogue-error", tenant))
                    .show(ui, |ui| DiagnosticReportView::new(error).show(ui));
                None
            }
        };
        if let Some(tenant) = tenant {
            if let Some(action) = self.tenant_manager.show_tenant(
                ui,
                tenant,
                &self.work_tracker,
                &self.executor,
                &self.work_cache,
                self.azure.authentication,
                accounts,
            ) {
                self.start_account_operation(action, None, ui.ctx());
            }
        } else if let Some(open) = self.tenant_manager.show_list(
            ui,
            &self.work_tracker,
            &self.executor,
            &self.work_cache,
            accounts,
        ) {
            open.apply(self);
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
        let Some(operation) = self.account_operation.take() else {
            return;
        };
        let origin = operation.origin;
        // Catalogue refresh resets inventory generations. Verify the originating
        // view before that reset so a late sign-in can't retarget another view.
        let picker_selection = self
            .azure_content_for_tab_mut(origin.tab)
            .filter(|content| content.account_generation == origin.generation)
            .and(origin.picker_selection);
        self.apply_account_catalogue(catalogue, provider_closed);
        match outcome {
            Ok(Some(new_account))
                if self
                    .tabs
                    .tab(origin.tab)
                    .is_some_and(|tab| tab.target().page() == PortalPage::Accounts) =>
            {
                self.account_manager.open_account(new_account);
            }
            Ok(Some(_)) => {}
            Ok(None) => {
                if let Some(selection) = picker_selection
                    && let Ok(catalogue) = &self.account_picker.catalogue
                    && selected_registration(catalogue, selection).is_some_and(|registration| {
                        matches!(
                            registration.session,
                            CloudTerrastodonAccountSession::Active { .. }
                        )
                    })
                {
                    self.select_account(origin.tab, selection);
                }
            }
            // Cleanup failures wrap the original cancellation and still need to
            // be shown. Only a bare cancellation is a normal dismissal.
            Err(error)
                if error
                    .report
                    .chain()
                    .next()
                    .is_some_and(|cause| cause.is::<CancellationError>()) => {}
            Err(error) => {
                self.show_account_error(origin.tab, operation.login_target.is_some(), error)
            }
        }
    }

    fn show_account_error(
        &mut self,
        origin: PortalTabId,
        signing_in: bool,
        report: DiagnosticReport,
    ) {
        // A sign-in prompt belongs to the app, which may display a different
        // tab by the time the external authentication flow finishes.
        if signing_in {
            self.account_sign_in_failure = Some(report);
            return;
        }
        match self.tabs.tab(origin).map(|tab| tab.target().page()) {
            Some(PortalPage::Accounts) => self.account_manager.show_error(report),
            Some(PortalPage::AzureTenants) => {
                self.account_picker
                    .set_selection_error(origin, Some(report));
            }
            Some(_) => {
                self.account_picker
                    .set_selection_error(origin, Some(report));
                self.account_picker.open(origin);
            }
            None => {}
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
                    self.account_picker.catalogue = Err(capture_gui_failure(
                        error,
                        "Could not install updated account authentication",
                    ));
                    self.account_profiles = Default::default();
                    self.invalidate_azure_accounts(|_| true);
                    return;
                }
            }
        }
        let previous = self.account_picker.catalogue.as_ref().ok().cloned();
        self.invalidate_azure_accounts(|content| {
            let (Some(previous), Ok(current)) = (&previous, &catalogue) else {
                return true;
            };
            // Requests not admitted by the retired provider must start again.
            (provider_closed && content.is_loading())
                || content.authentication.is_some_and(|selection| {
                    selected_registration(previous, selection)
                        != selected_registration(current, selection)
                })
                || content
                    .devops_pat_accounts
                    .values()
                    .any(|&id| pat_registration(previous, id) != pat_registration(current, id))
        });
        if let Ok(catalogue) = &catalogue {
            let missing = |selection| selected_registration(catalogue, selection).is_none();
            if self.egui_config.azure_authentication.is_some_and(missing) {
                self.egui_config.azure_authentication = None;
            }
            let tabs: Vec<_> = self.tabs.iter().map(|(_, tab, _)| tab.id).collect();
            for tab in tabs {
                if let Some(content) = self.azure_content_for_tab_mut(tab)
                    && content.authentication.is_some_and(missing)
                {
                    content.authentication = None;
                }
            }
        }
        self.account_picker.catalogue = catalogue;
        self.account_profiles = Default::default();
    }
}

fn pat_registration(
    catalogue: &CloudTerrastodonAccountCatalogue,
    account_id: CloudTerrastodonAccountId,
) -> Option<&CloudTerrastodonCredentialRegistration> {
    catalogue
        .by_id(account_id)
        .ok()?
        .registration(CloudTerrastodonCredentialSource::PersonalAccessToken)
        .ok()
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
