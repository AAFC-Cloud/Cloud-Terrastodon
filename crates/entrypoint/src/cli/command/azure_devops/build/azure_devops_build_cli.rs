use super::definition::AzureDevOpsBuildDefinitionArgs;
use super::list::AzureDevOpsBuildListArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;

/// Build runs, pipeline definitions, and definition folders.
#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsBuildArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsBuildCommand,
}

impl AzureDevOpsBuildArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<CliOutput> {
        self.command.invoke(auth_context).await
    }
}

/// Commands for build runs, pipeline definitions, and definition folders.
#[derive(Debug, Clone, facet::Facet)]
#[repr(u8)]
pub enum AzureDevOpsBuildCommand {
    /// List recent build runs in a project.
    List(AzureDevOpsBuildListArgs),
    /// List pipeline definitions and manage their folders.
    Definition(AzureDevOpsBuildDefinitionArgs),
}

impl AzureDevOpsBuildCommand {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<CliOutput> {
        match self {
            Self::List(args) => args.invoke(auth_context).await,
            Self::Definition(args) => args.invoke(auth_context).await,
        }
    }
}
