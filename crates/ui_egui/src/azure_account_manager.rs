use crate::azure_account_action::AzureAccountAction;
use crate::azure_account_confirmation::AzureAccountConfirmation;
use crate::azure_account_details::AzureAccountDetails;
use crate::azure_account_manager_page::AzureAccountManagerPage;
use cloud_terrastodon_auth::CloudTerrastodonAccountCatalogue;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialSource;
use eframe::egui::Context;
use eframe::egui::Id;
use eframe::egui::ScrollArea;
use eframe::egui::Ui;
use eframe::egui::Window;
use eyre::Result;
use teamy_tracing::DiagnosticReport;
use teamy_tracing::gui::DiagnosticReportView;

/// Manage local account registrations and their credentials from the GUI.
#[derive(Default)]
pub(crate) struct AzureAccountManager {
    open: bool,
    page: AzureAccountManagerPage,
    error: Option<DiagnosticReport>,
}

impl AzureAccountManager {
    pub fn is_open(&self) -> bool {
        self.open
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
        self.open = true;
        self.error = Some(error);
    }

    fn navigate(&mut self, page: AzureAccountManagerPage) {
        self.open = true;
        self.page = page;
        self.error = None;
    }

    pub fn show(
        &mut self,
        ctx: &Context,
        catalogue: Option<&CloudTerrastodonAccountCatalogue>,
        busy: bool,
    ) -> Option<AzureAccountAction> {
        if !self.open {
            return None;
        }

        let mut open = true;
        let mut action = None;
        Window::new("Accounts")
            .id(Id::new("account_manager"))
            .default_width(540.0)
            .open(&mut open)
            .show(ctx, |ui| {
                if let Some(error) = &self.error {
                    ui.collapsing(error.report.to_string(), |ui| {
                        DiagnosticReportView::new(error).show(ui);
                    });
                }
                let response = ui
                    .add_enabled_ui(!busy, |ui| {
                        ScrollArea::vertical()
                            .max_height(560.0)
                            .show(ui, |ui| self.show_page(ui, catalogue))
                            .inner
                    })
                    .inner;
                match response {
                    Ok(Some(next)) => {
                        self.error = None;
                        action = Some(next);
                    }
                    Ok(None) => {}
                    Err(error) => self.show_error(DiagnosticReport::new(error)),
                }
            });
        if !open {
            // Clear any credentials still being entered and abandon pending confirmations.
            *self = Self::default();
            return None;
        }
        action
    }

    fn show_page(
        &mut self,
        ui: &mut Ui,
        catalogue: Option<&CloudTerrastodonAccountCatalogue>,
    ) -> Result<Option<AzureAccountAction>> {
        if matches!(self.page, AzureAccountManagerPage::Accounts) {
            if ui.button("Add account").clicked() {
                self.open_add();
            }
        } else if ui.button("All accounts").clicked() {
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
                            ui.horizontal_wrapped(|ui| {
                                let open = ui.button(account.alias.as_str()).clicked();
                                ui.label(AzureAccountDetails::namespace_label(&account.namespace));
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
                    ui.label("Reload accounts from the account menu.");
                }
                Ok(None)
            }
            AzureAccountManagerPage::Add(form) => form.show(ui),
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
