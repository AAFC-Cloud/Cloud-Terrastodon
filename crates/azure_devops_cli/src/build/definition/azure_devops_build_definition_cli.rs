use super::folder::AzureDevOpsBuildDefinitionFolderArgs;
use super::list::AzureDevOpsBuildDefinitionListArgs;
use super::show::AzureDevOpsBuildDefinitionShowArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;

/// Pipeline definitions and their folders.
#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsBuildDefinitionArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsBuildDefinitionCommand,
}

impl AzureDevOpsBuildDefinitionArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<CliOutput> {
        self.command.invoke(auth_context, executor).await
    }
}

/// Commands for pipeline definitions and their folders.
#[derive(Debug, Clone, facet::Facet)]
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
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<CliOutput> {
        match self {
            Self::List(args) => args.invoke(auth_context, executor).await,
            Self::Show(args) => args.invoke(auth_context, executor).await,
            Self::Folder(args) => args.invoke(auth_context, executor).await,
        }
    }
}
