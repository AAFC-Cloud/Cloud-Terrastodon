use crate::interactive::browse_application_registrations;
use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use eyre::Result;

/// Interactively browse Entra (Azure AD) application registrations.
#[derive(Args, Debug, Clone)]
pub struct AzureEntraApplicationRegistrationBrowseArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureEntraApplicationRegistrationBrowseArgs {
    pub async fn invoke(self) -> Result<()> {
        browse_application_registrations(self.tenant.resolve().await?).await
    }
}
