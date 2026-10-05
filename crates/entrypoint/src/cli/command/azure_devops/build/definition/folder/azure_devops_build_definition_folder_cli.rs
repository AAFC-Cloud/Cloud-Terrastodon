use super::list::AzureDevOpsBuildDefinitionFolderListArgs;
use super::prune::AzureDevOpsBuildDefinitionFolderPruneArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Build definition folder listings and empty-folder pruning.
#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsBuildDefinitionFolderArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsBuildDefinitionFolderCommand,
}

impl AzureDevOpsBuildDefinitionFolderArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<CliOutput> {
        self.command.invoke(auth_context).await
    }
}

/// Commands for build definition folders.
#[derive(Debug, Clone, facet::Facet)]
#[repr(u8)]
pub enum AzureDevOpsBuildDefinitionFolderCommand {
    /// List build definition folders in a project.
    List(AzureDevOpsBuildDefinitionFolderListArgs),
    /// Prune empty folders; requires all definitions visible and no concurrent edits.
    /// Azure DevOps folder deletion can also delete definitions and their builds.
    Prune(AzureDevOpsBuildDefinitionFolderPruneArgs),
}

impl AzureDevOpsBuildDefinitionFolderCommand {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<CliOutput> {
        match self {
            Self::List(args) => args.invoke(auth_context).await,
            Self::Prune(args) => args.invoke(auth_context).await,
        }
    }
}
