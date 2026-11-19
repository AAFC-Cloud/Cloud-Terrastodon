use crate::AccountSignInPrompt;

/// How the account store should acquire its candidate credential.
pub(crate) enum AccountCredentialProvisioning<'a> {
    SignIn {
        personal_access_token: Option<&'a str>,
        prompt: &'a (dyn Fn(AccountSignInPrompt) + Send + Sync),
    },
    ImportLegacy,
}
