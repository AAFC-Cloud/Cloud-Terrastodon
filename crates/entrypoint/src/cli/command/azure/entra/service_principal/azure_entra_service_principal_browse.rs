use crate::interactive::prelude::browse_service_principals;
use clap::Args;
use eyre::Result;

/// Interactively browse Entra (Azure AD) service principals.
#[derive(Args, Debug, Clone)]
pub struct AzureEntraSpBrowseArgs {}

impl AzureEntraSpBrowseArgs {
    pub async fn invoke(self) -> Result<()> {
        browse_service_principals().await
    }
}
