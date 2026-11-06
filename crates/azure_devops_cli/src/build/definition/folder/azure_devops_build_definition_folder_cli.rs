use super::list::AzureDevOpsBuildDefinitionFolderListArgs;
use super::prune::AzureDevOpsBuildDefinitionFolderPruneArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Build definition folder listings and empty-folder pruning.
#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsBuildDefinitionFolderArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsBuildDefinitionFolderCommand,
}

impl AzureDevOpsBuildDefinitionFolderArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<CliOutput> {
        self.command.invoke(auth_context, executor).await
    }
}

/// Commands for build definition folders.
#[derive(Debug, Clone, Facet)]
#[repr(u8)]
pub enum AzureDevOpsBuildDefinitionFolderCommand {
    /// List build definition folders in a project.
    List(AzureDevOpsBuildDefinitionFolderListArgs),
    /// Prune empty folders; requires all definitions visible and no concurrent edits.
    /// Azure DevOps folder deletion can also delete definitions and their builds.
    Prune(AzureDevOpsBuildDefinitionFolderPruneArgs),
}

impl AzureDevOpsBuildDefinitionFolderCommand {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<CliOutput> {
        match self {
            Self::List(args) => args.invoke(auth_context, executor).await,
            Self::Prune(args) => args.invoke(auth_context, executor).await,
        }
    }
}
