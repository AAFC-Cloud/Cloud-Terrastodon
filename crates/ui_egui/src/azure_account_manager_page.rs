use crate::azure_account_details::AzureAccountDetails;
use crate::azure_account_registration_form::AzureAccountRegistrationForm;

#[derive(Default)]
pub(crate) enum AzureAccountManagerPage {
    #[default]
    Accounts,
    Add(AzureAccountRegistrationForm),
    Account(AzureAccountDetails),
}
