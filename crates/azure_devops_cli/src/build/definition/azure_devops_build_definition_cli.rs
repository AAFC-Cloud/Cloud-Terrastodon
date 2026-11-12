use super::folder::AzureDevOpsBuildDefinitionFolderArgs;
use super::list::AzureDevOpsBuildDefinitionListArgs;
use super::show::AzureDevOpsBuildDefinitionShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Pipeline definitions and their folders.
#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsBuildDefinitionArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsBuildDefinitionCommand,
}

impl AzureDevOpsBuildDefinitionArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        self.command.invoke(executor).await
    }
}

/// Commands for pipeline definitions and their folders.
#[derive(Debug, Clone, Facet)]
#[repr(u8)]
pub enum AzureDevOpsBuildDefinitionCommand {
    /// List pipeline definitions in a project.
    List(AzureDevOpsBuildDefinitionListArgs),
    /// Show a pipeline's full definition, including its repository and process.
    Show(AzureDevOpsBuildDefinitionShowArgs),
    /// List and prune definition folders.
    Folder(AzureDevOpsBuildDefinitionFolderArgs),
}

impl AzureDevOpsBuildDefinitionCommand {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        match self {
            Self::List(args) => Box::pin(args.invoke(executor)).await,
            Self::Show(args) => Box::pin(args.invoke(executor)).await,
            Self::Folder(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
