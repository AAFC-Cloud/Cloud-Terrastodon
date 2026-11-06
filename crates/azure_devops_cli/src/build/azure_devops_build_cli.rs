use super::definition::AzureDevOpsBuildDefinitionArgs;
use super::list::AzureDevOpsBuildListArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Build runs, pipeline definitions, and definition folders.
#[derive(Debug, Clone, Facet)]
pub struct AzureDevOpsBuildArgs {
    #[facet(figue::subcommand)]
    pub command: AzureDevOpsBuildCommand,
}

impl AzureDevOpsBuildArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<CliOutput> {
        self.command.invoke(auth_context, executor).await
    }
}

/// Commands for build runs, pipeline definitions, and definition folders.
#[derive(Debug, Clone, Facet)]
#[repr(u8)]
pub enum AzureDevOpsBuildCommand {
    /// List recent build runs in a project.
    List(AzureDevOpsBuildListArgs),
    /// List pipeline definitions and manage their folders.
    Definition(AzureDevOpsBuildDefinitionArgs),
}

impl AzureDevOpsBuildCommand {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<CliOutput> {
        match self {
            Self::List(args) => args.invoke(auth_context, executor).await,
            Self::Definition(args) => args.invoke(auth_context, executor).await,
        }
    }
}
