use crate::azure_account_action::AzureAccountAction;
use crate::azure_account_confirmation::AzureAccountConfirmation;
use crate::azure_account_registration_form::parse_client_id;
use crate::azure_account_registration_form::source_label;
use crate::azure_account_registration_form::text_input;
use crate::azure_account_sign_in_method::AzureAccountSignInMethod;
use cloud_terrastodon_auth::AzureDevOpsPersonalAccessToken;
use cloud_terrastodon_auth::CloudTerrastodonAccountCatalogue;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountCredential;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountNamespace;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountPrincipal;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountProfile;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountSession;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialRegistration;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialSource;
use eframe::egui::Align;
use eframe::egui::Button;
use eframe::egui::Key;
use eframe::egui::TextEdit;
use eframe::egui::Ui;
use eyre::Result;

/// An account detail view owns the credential drafts and confirmation for that account.
pub(crate) struct AzureAccountDetails {
    id: CloudTerrastodonAccountId,
    browser_client_id: Option<String>,
    personal_access_token: Option<String>,
    confirmation: Option<AzureAccountConfirmation>,
}

impl AzureAccountDetails {
    pub(crate) fn account_id(&self) -> CloudTerrastodonAccountId {
        self.id
    }

    pub fn new(id: CloudTerrastodonAccountId) -> Self {
        Self {
            id,
            browser_client_id: None,
            personal_access_token: None,
            confirmation: None,
        }
    }

    pub fn with_confirmation(
        id: CloudTerrastodonAccountId,
        confirmation: AzureAccountConfirmation,
    ) -> Self {
        Self {
            confirmation: Some(confirmation),
            ..Self::new(id)
        }
    }

    pub fn clear_drafts(&mut self) {
        self.browser_client_id = None;
        self.personal_access_token = None;
        self.confirmation = None;
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        catalogue: &CloudTerrastodonAccountCatalogue,
    ) -> Result<Option<AzureAccountAction>> {
        let Ok(account) = catalogue.by_id(self.id) else {
            self.clear_drafts();
            ui.label("This account is no longer registered.");
            return Ok(None);
        };
        ui.heading(account.alias.as_str());
        ui.label(Self::namespace_label(&account.namespace));
        ui.label(format!("Account ID · {}", account.id));
        match account.principal {
            Some(CloudTerrastodonAccountPrincipal::EntraUser(id)) => {
                ui.label(format!("Entra user · {id}"));
            }
            Some(CloudTerrastodonAccountPrincipal::EntraApplication(id)) => {
                ui.label(format!("Entra application · {id}"));
            }
            None => {}
        }

        if let Some(confirmation) = self.confirmation
            && confirmation.is_available(account)
        {
            return Ok(confirmation.show(ui, account.id, &mut self.confirmation));
        }
        self.confirmation = None;

        let is_default = catalogue
            .defaults
            .iter()
            .any(|default| default.account_id == account.id);
        let account_action = ui
            .horizontal_wrapped(|ui| {
                if is_default {
                    ui.label("Default account");
                    if ui.button("Clear default").clicked() {
                        return Some(AzureAccountAction::ClearDefault(account.id));
                    }
                } else if ui.button("Set as default").clicked() {
                    return Some(AzureAccountAction::SetDefault(account.id));
                }
                if account.credentials.len() > 1
                    && account.credentials.iter().any(|registration| {
                        !matches!(
                            registration.session,
                            CloudTerrastodonAccountSession::Unprovisioned
                        )
                    })
                    && ui.button("Sign out of all sources").clicked()
                {
                    self.confirmation = Some(AzureAccountConfirmation::SignOut(None));
                }
                if ui.button("Remove account").clicked() {
                    self.confirmation = Some(AzureAccountConfirmation::Remove);
                }
                None
            })
            .inner;
        if account_action.is_some() || self.confirmation.is_some() {
            self.personal_access_token = None;
            return Ok(account_action);
        }

        ui.separator();
        ui.heading("Authentication sources");
        for registration in &account.credentials {
            let action = ui
                .push_id(registration.credential.source(), |ui| {
                    self.show_credential(ui, account, registration)
                })
                .inner;
            if action.is_some() || self.confirmation.is_some() {
                self.personal_access_token = None;
                return Ok(action);
            }
            ui.separator();
        }

        self.show_add_credential(ui, account)
    }

    fn show_credential(
        &mut self,
        ui: &mut Ui,
        account: &CloudTerrastodonAccountProfile,
        registration: &CloudTerrastodonCredentialRegistration,
    ) -> Option<AzureAccountAction> {
        use CloudTerrastodonAccountSession as Session;
        use CloudTerrastodonCredentialSource as Source;
        let source = registration.credential.source();
        let entra = matches!(
            account.namespace,
            CloudTerrastodonAccountNamespace::Entra { .. }
        );
        let active = matches!(registration.session, Session::Active { .. });
        let can_sign_in =
            !active && (entra || !matches!(registration.session, Session::Retired { .. }));
        if source == Source::PersonalAccessToken && !can_sign_in {
            self.personal_access_token = None;
        }

        ui.horizontal(|ui| {
            ui.strong(source_label(source));
            if source == account.preferred_source {
                ui.label("Preferred");
            }
            ui.label(match registration.session {
                Session::Unprovisioned => "Signed out",
                Session::Provisioning { .. } => "Sign-in incomplete",
                Session::Active { .. } => "Saved session",
                Session::Retired { .. } => "Session retired",
            });
        });
        match &registration.credential {
            CloudTerrastodonAccountCredential::Browser { client_id, .. } => {
                ui.label(format!("Application client ID · {client_id}"));
            }
            CloudTerrastodonAccountCredential::WorkloadIdentity {
                client_id,
                assertion_file,
                ..
            } => {
                ui.label(format!("Application client ID · {client_id}"));
                ui.label(format!(
                    "Assertion file · {}",
                    assertion_file.as_ref().display()
                ));
            }
            CloudTerrastodonAccountCredential::Gitea {
                login,
                configuration,
                ..
            } => {
                ui.label(format!("Tea login · {login}"));
                ui.label(format!(
                    "Tea configuration · {}",
                    configuration.as_ref().display()
                ));
            }
            _ => {}
        }
        let mut open_token_input = false;
        let action = ui
            .horizontal_wrapped(|ui| {
                if source != account.preferred_source && ui.button("Make preferred").clicked() {
                    return Some(AzureAccountAction::Prefer {
                        account_id: account.id,
                        source,
                    });
                }
                if active && entra && ui.button("Sign in again").clicked() {
                    self.confirmation = Some(AzureAccountConfirmation::Reauthenticate(source));
                }
                if !matches!(registration.session, Session::Unprovisioned)
                    && ui
                        .button(if active {
                            "Sign out"
                        } else {
                            "Clean up sign-in"
                        })
                        .clicked()
                {
                    self.confirmation = Some(AzureAccountConfirmation::SignOut(Some(source)));
                }
                if can_sign_in
                    && ui
                        .add_enabled(self.personal_access_token.is_none(), Button::new("Sign in"))
                        .clicked()
                {
                    if source == Source::PersonalAccessToken {
                        open_token_input = true;
                    } else {
                        return Some(AzureAccountAction::sign_in(account.id, source));
                    }
                }
                if can_sign_in
                    && source == Source::AzureCli
                    && ui.button("Sign in with device code").clicked()
                {
                    return Some(AzureAccountAction::Login {
                        account_id: account.id,
                        method: AzureAccountSignInMethod::AzureCliDeviceCode,
                    });
                }
                if can_sign_in
                    && matches!(
                        source,
                        Source::Browser | Source::AzureCli | Source::PersonalAccessToken
                    )
                    && ui.button("Import legacy sign-in").clicked()
                {
                    return Some(AzureAccountAction::ImportLegacy {
                        account_id: account.id,
                        source,
                    });
                }
                None
            })
            .inner;
        if action.is_some() || self.confirmation.is_some() {
            return action;
        }
        if !entra && matches!(registration.session, Session::Retired { .. }) {
            ui.label("Register a new account to replace this credential.");
        }
        if open_token_input {
            self.personal_access_token = Some(String::new());
        }
        if source == Source::PersonalAccessToken {
            return self.show_pat_login(ui, open_token_input);
        }
        None
    }

    fn show_pat_login(&mut self, ui: &mut Ui, request_focus: bool) -> Option<AzureAccountAction> {
        let token = self.personal_access_token.as_mut()?;
        let label = ui.label("Personal access token");
        let mut edit = TextEdit::singleline(token).password(true).show(ui);
        let response = edit.response.response.labelled_by(label.id);
        // egui's undo history can outlive the password field and contains the input text.
        edit.state.clear_undoer();
        if request_focus {
            response.request_focus();
            response.scroll_to_me(Some(Align::Center));
        }
        let has_token = !token.trim().is_empty();
        let enter = response.lost_focus() && ui.input(|input| input.key_pressed(Key::Enter));
        let (submit, cancel) = ui
            .horizontal(|ui| {
                (
                    ui.add_enabled(has_token, Button::new("Save and sign in"))
                        .clicked(),
                    ui.button("Cancel").clicked(),
                )
            })
            .inner;
        if cancel {
            self.personal_access_token = None;
        } else if has_token && (submit || enter) {
            let mut token = self.personal_access_token.take()?;
            token.truncate(token.trim_end().len());
            return Some(AzureAccountAction::SavePersonalAccessToken {
                account_id: self.id,
                token: AzureDevOpsPersonalAccessToken::new(token),
            });
        }
        None
    }

    fn show_add_credential(
        &mut self,
        ui: &mut Ui,
        account: &CloudTerrastodonAccountProfile,
    ) -> Result<Option<AzureAccountAction>> {
        use CloudTerrastodonCredentialSource as Source;
        let CloudTerrastodonAccountNamespace::Entra { tenant_id } = account.namespace else {
            return Ok(None);
        };
        let credential = match account.preferred_source {
            Source::Browser
                if account.registration(Source::AzureCli).is_err()
                    && ui.button("Add Azure CLI authentication").clicked() =>
            {
                CloudTerrastodonAccountCredential::AzureCli { tenant_id }
            }
            Source::AzureCli if account.registration(Source::Browser).is_err() => {
                if ui.button("Add browser authentication").clicked() {
                    self.browser_client_id = Some(String::new());
                }
                let Some(client_id) = &mut self.browser_client_id else {
                    return Ok(None);
                };
                text_input(ui, "Application client ID", client_id);
                let (save, cancel) = ui
                    .horizontal(|ui| {
                        (
                            ui.button("Register browser authentication").clicked(),
                            ui.button("Cancel").clicked(),
                        )
                    })
                    .inner;
                if cancel {
                    self.browser_client_id = None;
                    return Ok(None);
                }
                if !save {
                    return Ok(None);
                }
                let client_id = parse_client_id(client_id)?;
                self.browser_client_id = None;
                CloudTerrastodonAccountCredential::Browser {
                    tenant_id,
                    client_id,
                }
            }
            _ => return Ok(None),
        };
        Ok(Some(AzureAccountAction::RegisterCredential {
            account_id: account.id,
            credential,
        }))
    }

    pub fn namespace_label(namespace: &CloudTerrastodonAccountNamespace) -> String {
        match namespace {
            CloudTerrastodonAccountNamespace::Entra { tenant_id } => {
                format!("Tenant · {tenant_id}")
            }
            CloudTerrastodonAccountNamespace::AzureDevOps { organization } => {
                format!("Azure DevOps · {organization}")
            }
            CloudTerrastodonAccountNamespace::Gitea { instance } => format!("Gitea · {instance}"),
        }
    }
}
