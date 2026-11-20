/// How Azure CLI should start an interactive login for the selected tenant.
#[derive(Clone, Copy)]
pub(crate) enum AzureCliSignInMode {
    Automatic,
    DeviceCode,
}
