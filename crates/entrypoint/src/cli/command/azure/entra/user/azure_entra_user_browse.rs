use clap::Args;
use crate::interactive::prelude::browse_users;
use eyre::Result;

/// Interactively browse Entra (Azure AD) users.
#[derive(Args, Debug, Clone)]
pub struct AzureEntraUserBrowseArgs {}

impl AzureEntraUserBrowseArgs {
    pub async fn invoke(self) -> Result<()> {
        browse_users().await
    }
}
