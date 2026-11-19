use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;

/// A selection or management action initiated from the account menu.
pub(crate) enum AzureAccountPickerAction {
    Select(AzureBearerAuthenticationSelection),
    SignIn(AzureBearerAuthenticationSelection),
    Reauthenticate(AzureBearerAuthenticationSelection),
    Manage(CloudTerrastodonAccountId),
    Add,
    Reload,
    ManageAccounts,
    ManageTenants,
}
