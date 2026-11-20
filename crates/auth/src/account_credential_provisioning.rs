use crate::AccountSignInPrompt;
use crate::azure_cli_sign_in_mode::AzureCliSignInMode;

/// How the account store should acquire its candidate credential.
pub(crate) enum AccountCredentialProvisioning<'a> {
    SignIn {
        personal_access_token: Option<&'a str>,
        prompt: &'a (dyn Fn(AccountSignInPrompt) + Send + Sync),
        azure_cli_mode: AzureCliSignInMode,
    },
    ImportLegacy,
}
