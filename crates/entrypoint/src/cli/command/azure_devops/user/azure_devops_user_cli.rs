use crate::cli::azure_devops::user::AzureDevOpsUserUpdateTuiArgs;
use crate::cli::azure_devops::user::update::AzureDevOpsUserUpdateArgs;
use clap::Args;
use clap::Subcommand;
use eyre::Result;

/// Azure DevOps user-related commands.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsUserArgs {
    #[command(subcommand)]
    pub command: AzureDevOpsUserCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AzureDevOpsUserCommand {
    /// User-specific operations.
    Update(AzureDevOpsUserUpdateArgs),
    /// User-specific operations.
    UpdateTui(AzureDevOpsUserUpdateTuiArgs),
}

impl AzureDevOpsUserArgs {
    pub async fn invoke(self) -> Result<()> {
        match self.command {
            AzureDevOpsUserCommand::Update(args) => args.invoke().await?,
            AzureDevOpsUserCommand::UpdateTui(args) => args.invoke().await?,
        }

        Ok(())
    }
}
