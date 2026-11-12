use super::account_add_cli::AccountAddArgs;
use super::account_credential_cli::AccountCredentialArgs;
use super::account_default_cli::AccountDefaultArgs;
use super::account_import_legacy_cli::AccountImportLegacyArgs;
use super::account_list_cli::AccountListArgs;
use super::account_login_cli::AccountLoginArgs;
use super::account_logout_cli::AccountLogoutArgs;
use super::account_remove_cli::AccountRemoveArgs;
use super::account_show_cli::AccountShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Manage explicitly selected Cloud Terrastodon accounts and credentials.
#[derive(Debug, Facet)]
pub struct AccountArgs {
    #[facet(figue::subcommand)]
    pub command: AccountCommand,
}
#[derive(Debug, Facet)]
#[repr(u8)]
pub enum AccountCommand {
    Add(AccountAddArgs),
    Credential(AccountCredentialArgs),
    ImportLegacy(AccountImportLegacyArgs),
    List(AccountListArgs),
    Show(AccountShowArgs),
    Login(AccountLoginArgs),
    Logout(AccountLogoutArgs),
    Remove(AccountRemoveArgs),
    Default(AccountDefaultArgs),
}
impl AccountArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self.command {
            AccountCommand::Add(args) => Box::pin(args.invoke(executor.cancellation())).await,
            AccountCommand::Credential(args) => {
                Box::pin(args.invoke(executor.cancellation())).await
            }
            AccountCommand::ImportLegacy(args) => {
                Box::pin(args.invoke(executor.cancellation())).await
            }
            AccountCommand::List(args) => Box::pin(args.invoke(executor.cancellation())).await,
            AccountCommand::Show(args) => Box::pin(args.invoke(executor.cancellation())).await,
            AccountCommand::Login(args) => Box::pin(args.invoke(executor.cancellation())).await,
            AccountCommand::Logout(args) => Box::pin(args.invoke(executor)).await,
            AccountCommand::Remove(args) => Box::pin(args.invoke(executor)).await,
            AccountCommand::Default(args) => Box::pin(args.invoke(executor.cancellation())).await,
        }
    }
}
