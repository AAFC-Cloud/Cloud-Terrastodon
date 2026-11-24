use super::AzureDevOpsOrganizationAddArgs;
use super::AzureDevOpsOrganizationListArgs;
use super::AzureDevOpsOrganizationRemoveArgs;
use super::AzureDevOpsOrganizationShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use facet::Facet;

/// Manage Azure DevOps organizations tracked on this device.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsOrganizationArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsOrganizationCommand,
}

#[derive(Facet, Debug, Clone)]
#[repr(u8)]
pub enum AzureDevOpsOrganizationCommand {
    /// List organizations tracked on this device.
    List(AzureDevOpsOrganizationListArgs),
    /// Track an organization or replace its tenant association.
    Add(AzureDevOpsOrganizationAddArgs),
    /// Show an organization's local registration.
    Show(AzureDevOpsOrganizationShowArgs),
    /// Remove an organization's local registration.
    Remove(AzureDevOpsOrganizationRemoveArgs),
}

impl AzureDevOpsOrganizationArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        match self.command {
            AzureDevOpsOrganizationCommand::List(args) => Box::pin(args.invoke()).await,
            AzureDevOpsOrganizationCommand::Add(args) => Box::pin(args.invoke(cancellation)).await,
            AzureDevOpsOrganizationCommand::Show(args) => Box::pin(args.invoke()).await,
            AzureDevOpsOrganizationCommand::Remove(args) => {
                Box::pin(args.invoke(cancellation)).await
            }
        }
    }
}
