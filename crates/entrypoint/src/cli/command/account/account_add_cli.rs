use super::account_add_azure_cli_cli::AccountAddAzureCliArgs;
use super::account_add_azure_devops_cli::AccountAddAzureDevOpsArgs;
use super::account_add_gitea_cli::AccountAddGiteaArgs;
use super::account_add_workload_identity_cli::AccountAddWorkloadIdentityArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AccountAddBrowserArgs;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use facet::Facet;

/// Register a local account without acquiring credentials.
#[derive(Debug, Facet)]
pub struct AccountAddArgs {
    #[facet(figue::subcommand)]
    pub command: AccountAddCommand,
}
#[derive(Debug, Facet)]
#[repr(u8)]
pub enum AccountAddCommand {
    AzureCli(AccountAddAzureCliArgs),
    Browser(AccountAddBrowserArgs),
    WorkloadIdentity(AccountAddWorkloadIdentityArgs),
    AzureDevOps(AccountAddAzureDevOpsArgs),
    Gitea(AccountAddGiteaArgs),
}
impl AccountAddArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        match self.command {
            AccountAddCommand::AzureCli(args) => Box::pin(args.invoke(cancellation)).await,
            AccountAddCommand::Browser(args) => Box::pin(args.invoke(cancellation)).await,
            AccountAddCommand::WorkloadIdentity(args) => Box::pin(args.invoke(cancellation)).await,
            AccountAddCommand::AzureDevOps(args) => Box::pin(args.invoke(cancellation)).await,
            AccountAddCommand::Gitea(args) => Box::pin(args.invoke(cancellation)).await,
        }
    }
}
