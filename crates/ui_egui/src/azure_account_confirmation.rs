use crate::azure_account_action::AzureAccountAction;
use crate::azure_account_registration_form::source_label;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountProfile;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountSession;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialSource;
use eframe::egui::Ui;

/// Operations that can retire a session or remove a profile require confirmation.
#[derive(Clone, Copy)]
pub(crate) enum AzureAccountConfirmation {
    Remove,
    SignOut(Option<CloudTerrastodonCredentialSource>),
    Reauthenticate(CloudTerrastodonCredentialSource),
}

impl AzureAccountConfirmation {
    pub fn is_available(self, account: &CloudTerrastodonAccountProfile) -> bool {
        match self {
            Self::Reauthenticate(source) => {
                account.registration(source).is_ok_and(|registration| {
                    matches!(
                        registration.session,
                        CloudTerrastodonAccountSession::Active { .. }
                    )
                })
            }
            Self::SignOut(Some(source)) => account.registration(source).is_ok(),
            _ => true,
        }
    }

    pub fn show(
        self,
        ui: &mut Ui,
        account_id: CloudTerrastodonAccountId,
        pending: &mut Option<Self>,
    ) -> Option<AzureAccountAction> {
        let (question, button, action) = match self {
            Self::Remove => (
                "Remove this account and its local credentials?".to_owned(),
                "Remove account",
                AzureAccountAction::Remove(account_id),
            ),
            Self::SignOut(source) => (
                source.map_or_else(
                    || "Sign out of every authentication source for this account?".to_owned(),
                    |source| format!("Retire the saved {} sign-in?", source_label(source)),
                ),
                "Sign out",
                AzureAccountAction::Logout { account_id, source },
            ),
            Self::Reauthenticate(source) => (
                format!(
                    "Retire the saved {} session and sign in again as the same identity?",
                    source_label(source)
                ),
                "Replace session",
                AzureAccountAction::Reauthenticate { account_id, source },
            ),
        };
        ui.label(question);
        ui.horizontal(|ui| {
            if ui.button(button).clicked() {
                *pending = None;
                return Some(action);
            }
            if ui.button("Cancel").clicked() {
                *pending = None;
            }
            None
        })
        .inner
    }
}
