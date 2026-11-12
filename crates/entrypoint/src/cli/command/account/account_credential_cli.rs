use super::account_credential_add_azure_cli_cli::AccountCredentialAddAzureCliArgs;
use super::account_credential_add_browser_cli::AccountCredentialAddBrowserArgs;
use super::account_credential_prefer_cli::AccountCredentialPreferArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use facet::Facet;

/// Register another mechanism or select an account's preferred credential source.
#[derive(Debug, Facet)]
pub struct AccountCredentialArgs {
    #[facet(figue::subcommand)]
    pub command: AccountCredentialCommand,
}
#[derive(Debug, Facet)]
#[repr(u8)]
pub enum AccountCredentialCommand {
    AddAzureCli(AccountCredentialAddAzureCliArgs),
    AddBrowser(AccountCredentialAddBrowserArgs),
    Prefer(AccountCredentialPreferArgs),
}
impl AccountCredentialArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        match self.command {
            AccountCredentialCommand::AddAzureCli(args) => {
                Box::pin(args.invoke(cancellation)).await
            }
            AccountCredentialCommand::AddBrowser(args) => Box::pin(args.invoke(cancellation)).await,
            AccountCredentialCommand::Prefer(args) => Box::pin(args.invoke(cancellation)).await,
        }
    }
}
