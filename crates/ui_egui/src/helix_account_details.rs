use crate::azure_account_action::AzureAccountAction;
use crate::azure_account_details::AzureAccountDetails;
use cloud_terrastodon_auth::CloudTerrastodonAccountCatalogue;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountSession;
use cloud_terrastodon_bmc_helix::BmcHelixServerCatalog;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerUrl;
use eframe::egui::Align;
use eframe::egui::Button;
use eframe::egui::ComboBox;
use eframe::egui::Key;
use eframe::egui::TextEdit;
use eframe::egui::Ui;

/// Server choice and private drafts for managing one Helix account's sessions.
#[derive(Default)]
pub(crate) struct HelixAccountDetails {
    server: Option<BmcHelixServerUrl>,
    password: Option<String>,
    confirm_logout: bool,
}

impl HelixAccountDetails {
    pub fn new(server: BmcHelixServerUrl) -> Self {
        Self {
            server: Some(server),
            ..Default::default()
        }
    }

    pub fn clear_drafts(&mut self) {
        self.password = None;
        self.confirm_logout = false;
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        account_id: CloudTerrastodonAccountId,
        accounts: &CloudTerrastodonAccountCatalogue,
        servers: Option<&BmcHelixServerCatalog>,
    ) -> Option<AzureAccountAction> {
        let Some(servers) = servers else {
            self.clear_drafts();
            return None;
        };
        if servers.servers.is_empty() {
            self.clear_drafts();
            ui.label("Add an endpoint in Helix servers to sign in.");
            return None;
        }
        let mut selected = self
            .server
            .clone()
            .filter(|server| {
                servers
                    .servers
                    .values()
                    .any(|candidate| candidate == server)
            })
            .or_else(|| {
                servers
                    .default
                    .as_ref()
                    .and_then(|name| servers.servers.get(name))
                    .cloned()
            });
        ui.horizontal(|ui| {
            ui.label("Server");
            let selected_name = selected.as_ref().and_then(|server| {
                servers
                    .servers
                    .iter()
                    .find(|(_, url)| *url == server)
                    .map(|(name, _)| name.as_str())
            });
            ComboBox::from_id_salt(("helix-session-server", account_id))
                .selected_text(selected_name.unwrap_or("Choose server"))
                .show_ui(ui, |ui| {
                    for (name, server) in &servers.servers {
                        ui.selectable_value(&mut selected, Some(server.clone()), name.as_str());
                    }
                });
        });
        if selected != self.server {
            self.clear_drafts();
            self.server = selected;
        }
        let server = self.server.clone()?;
        let session = accounts.bmc_helix_session(account_id, &server);
        let active = matches!(session, CloudTerrastodonAccountSession::Active { .. });
        ui.horizontal_wrapped(|ui| {
            ui.label(server.as_url().as_str());
            ui.label(AzureAccountDetails::session_label(session));
        });

        if self.confirm_logout {
            ui.label("Retire the saved Helix session for this server?");
            return ui
                .horizontal(|ui| {
                    if ui.button("Sign out").clicked() {
                        self.clear_drafts();
                        return Some(AzureAccountAction::SignOutOfHelix { account_id, server });
                    }
                    if ui.button("Cancel").clicked() {
                        self.clear_drafts();
                    }
                    None
                })
                .inner;
        }
        if self.password.is_some() {
            return self.show_password(ui, account_id, &server, active, false);
        }

        let is_default = accounts
            .resolve_bmc_helix(&server, None)
            .is_ok_and(|account| account.id == account_id);
        let (action, enter_password) = ui
            .horizontal_wrapped(|ui| {
                if is_default {
                    if ui.button("Clear default for this server").clicked() {
                        return (
                            Some(AzureAccountAction::ClearHelixDefault(server.clone())),
                            false,
                        );
                    }
                } else if ui.button("Set as default for this server").clicked() {
                    return (
                        Some(AzureAccountAction::SetHelixDefault {
                            account_id,
                            server: server.clone(),
                        }),
                        false,
                    );
                }
                if !matches!(session, CloudTerrastodonAccountSession::Unprovisioned)
                    && ui
                        .button(if active {
                            "Sign out"
                        } else {
                            "Clean up sign-in"
                        })
                        .clicked()
                {
                    self.confirm_logout = true;
                    return (None, false);
                }
                (
                    None,
                    ui.button(if active { "Sign in again" } else { "Sign in" })
                        .clicked(),
                )
            })
            .inner;
        if enter_password {
            self.password = Some(String::new());
            self.show_password(ui, account_id, &server, active, true)
        } else {
            action
        }
    }

    fn show_password(
        &mut self,
        ui: &mut Ui,
        account_id: CloudTerrastodonAccountId,
        server: &BmcHelixServerUrl,
        reauthenticate: bool,
        request_focus: bool,
    ) -> Option<AzureAccountAction> {
        let password = self.password.as_mut()?;
        let label = ui.label("Helix password");
        let mut edit = TextEdit::singleline(password).password(true).show(ui);
        // The persisted text edit state shares its undo history with this output.
        edit.state.clear_undoer();
        let response = edit.response.response.labelled_by(label.id);
        if request_focus {
            response.request_focus();
            response.scroll_to_me(Some(Align::Center));
        }
        let can_sign_in = !password.is_empty();
        let enter = response.lost_focus() && ui.input(|input| input.key_pressed(Key::Enter));
        let (submit, cancel) = ui
            .horizontal(|ui| {
                (
                    ui.add_enabled(can_sign_in, Button::new("Sign in"))
                        .clicked(),
                    ui.button("Cancel").clicked(),
                )
            })
            .inner;
        if cancel {
            self.clear_drafts();
        } else if can_sign_in && (submit || enter) {
            return Some(AzureAccountAction::SignInToHelix {
                account_id,
                server: server.clone(),
                password: self.password.take()?,
                reauthenticate,
            });
        }
        None
    }
}
