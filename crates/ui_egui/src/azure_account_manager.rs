use crate::azure_account_action::AzureAccountAction;
use crate::azure_account_confirmation::AzureAccountConfirmation;
use crate::azure_account_details::AzureAccountDetails;
use crate::azure_account_manager_page::AzureAccountManagerPage;
use cloud_terrastodon_auth::CloudTerrastodonAccountCatalogue;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialSource;
use cloud_terrastodon_azure::AzureTenantAlias;
use cloud_terrastodon_azure::AzureTenantId;
use eframe::egui::Frame;
use eframe::egui::ScrollArea;
use eframe::egui::Ui;
use eyre::Result;
use teamy_tracing::DiagnosticReport;
use teamy_tracing::gui::DiagnosticReportView;

/// Manage local account registrations and their credentials from the GUI.
#[derive(Default)]
pub(crate) struct AzureAccountManager {
    page: AzureAccountManagerPage,
    error: Option<DiagnosticReport>,
}

impl AzureAccountManager {
    pub(crate) fn cli_command(&self) -> Option<String> {
        match &self.page {
            AzureAccountManagerPage::Accounts => Some("ct account list".into()),
            AzureAccountManagerPage::Account(details) => {
                Some(format!("ct account show {}", details.account_id()))
            }
            AzureAccountManagerPage::Add(_) => None,
        }
    }

    pub fn open(&mut self) {
        self.navigate(AzureAccountManagerPage::Accounts);
    }

    pub fn open_add(&mut self) {
        self.navigate(AzureAccountManagerPage::Add(Default::default()));
    }

    pub fn open_account(&mut self, id: CloudTerrastodonAccountId) {
        self.navigate(AzureAccountManagerPage::Account(AzureAccountDetails::new(
            id,
        )));
    }

    pub fn open_reauthentication(
        &mut self,
        id: CloudTerrastodonAccountId,
        source: CloudTerrastodonCredentialSource,
    ) {
        self.navigate(AzureAccountManagerPage::Account(
            AzureAccountDetails::with_confirmation(
                id,
                AzureAccountConfirmation::Reauthenticate(source),
            ),
        ));
    }

    pub fn show_error(&mut self, error: DiagnosticReport) {
        self.error = Some(error);
    }

    fn navigate(&mut self, page: AzureAccountManagerPage) {
        self.page = page;
        self.error = None;
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        catalogue: Option<&CloudTerrastodonAccountCatalogue>,
        busy: bool,
        tracked_tenants: &[(AzureTenantId, Vec<AzureTenantAlias>)],
        open_tenants: &mut bool,
    ) -> Option<AzureAccountAction> {
        ScrollArea::vertical()
            .id_salt("account-management")
            .show(ui, |ui| {
                ui.set_max_width(ui.available_width().min(820.0));
                if let Some(error) = &self.error {
                    ui.collapsing(error.report.to_string(), |ui| {
                        DiagnosticReportView::new(error).show(ui);
                    });
                }
                let response = ui
                    .add_enabled_ui(!busy, |ui| {
                        self.show_page(ui, catalogue, tracked_tenants, open_tenants)
                    })
                    .inner;
                match response {
                    Ok(Some(next)) => {
                        self.error = None;
                        Some(next)
                    }
                    Ok(None) => None,
                    Err(error) => {
                        self.show_error(DiagnosticReport::new(error));
                        None
                    }
                }
            })
            .inner
    }

    fn show_page(
        &mut self,
        ui: &mut Ui,
        catalogue: Option<&CloudTerrastodonAccountCatalogue>,
        tracked_tenants: &[(AzureTenantId, Vec<AzureTenantAlias>)],
        open_tenants: &mut bool,
    ) -> Result<Option<AzureAccountAction>> {
        if matches!(self.page, AzureAccountManagerPage::Accounts) {
            let reload = ui
                .horizontal_wrapped(|ui| {
                    if ui.button("Add account").clicked() {
                        self.open_add();
                    }
                    if ui.button("Tenants").clicked() {
                        *open_tenants = true;
                    }
                    ui.button("Reload").clicked()
                })
                .inner;
            if reload {
                return Ok(Some(AzureAccountAction::Reload));
            }
            ui.add_space(8.0);
        } else if ui.link("Back to accounts").clicked() {
            self.open();
        }

        match &mut self.page {
            AzureAccountManagerPage::Accounts => {
                if let Some(catalogue) = catalogue {
                    if catalogue.accounts.is_empty() {
                        ui.label("No accounts registered.");
                    }
                    for account in &catalogue.accounts {
                        let open = ui.push_id(account.id, |ui| {
                            Frame::group(ui.style())
                                .show(ui, |ui| {
                                    ui.set_min_width(ui.available_width());
                                    let open = ui.link(account.alias.as_str()).clicked();
                                    ui.small(AzureAccountDetails::namespace_label(
                                        &account.namespace,
                                    ));
                                    open
                                })
                                .inner
                        });
                        if open.inner {
                            self.open_account(account.id);
                            break;
                        }
                    }
                } else {
                    ui.label("Could not load accounts. Select Reload to try again.");
                }
                Ok(None)
            }
            AzureAccountManagerPage::Add(form) => {
                ui.heading("Add account");
                form.show(ui, tracked_tenants, open_tenants)
            }
            AzureAccountManagerPage::Account(details) => {
                let Some(catalogue) = catalogue else {
                    // Do not retain a secret while the catalogue is unavailable.
                    details.clear_drafts();
                    return Ok(None);
                };
                details.show(ui, catalogue)
            }
        }
    }
}
