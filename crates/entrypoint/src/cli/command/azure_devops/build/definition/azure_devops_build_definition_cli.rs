use super::folder::AzureDevOpsBuildDefinitionFolderArgs;
use super::list::AzureDevOpsBuildDefinitionListArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Pipeline definitions and their folders.
#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsBuildDefinitionArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsBuildDefinitionCommand,
}

impl AzureDevOpsBuildDefinitionArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<CliOutput> {
        self.command.invoke(auth_context).await
    }
}

/// Commands for pipeline definitions and their folders.
#[derive(Debug, Clone, facet::Facet)]
#[repr(u8)]
pub enum AzureDevOpsBuildDefinitionCommand {
    /// List pipeline definitions in a project.
    List(AzureDevOpsBuildDefinitionListArgs),
    /// List and prune definition folders.
    Folder(AzureDevOpsBuildDefinitionFolderArgs),
}

impl AzureDevOpsBuildDefinitionCommand {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<CliOutput> {
        match self {
            Self::List(args) => args.invoke(auth_context).await,
            Self::Folder(args) => args.invoke(auth_context).await,
        }
    }
}
