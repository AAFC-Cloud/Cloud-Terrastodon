use crate::ResourceTypesCheckArgs;
use crate::ResourceTypesGenerateArgs;
use crate::ResourceTypesRefreshArgs;
use cloud_terrastodon_app::ApplicationCli;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_app::GlobalArgs;
use cloud_terrastodon_app::Result;
use cloud_terrastodon_app::facet;
use cloud_terrastodon_app::figue;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;

/// Refresh tenant metadata or maintain source from a reviewed local catalogue.
#[derive(facet::Facet)]
pub struct ResourceTypesArgs {
    #[facet(flatten)]
    pub global_args: GlobalArgs,
    #[facet(figue::subcommand)]
    pub command: ResourceTypesCommand,
}

impl ApplicationCli for ResourceTypesArgs {
    fn global_args(&self) -> &GlobalArgs {
        &self.global_args
    }
}

impl ResourceTypesArgs {
    pub async fn invoke(&self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let cancellation = executor.cancellation();
        match &self.command {
            ResourceTypesCommand::Generate(args) => args.invoke(cancellation),
            ResourceTypesCommand::Check(args) => args.invoke(cancellation),
            ResourceTypesCommand::Refresh(args) => args.invoke(executor).await,
        }
    }
}

#[derive(facet::Facet)]
#[repr(u8)]
pub enum ResourceTypesCommand {
    /// Write the generated enum from a local JSON array of resource-type names.
    Generate(ResourceTypesGenerateArgs),
    /// Verify generated source matches a local catalogue without modifying files.
    Check(ResourceTypesCheckArgs),
    /// Fetch provider metadata for the selected tenant and save a local catalogue.
    Refresh(ResourceTypesRefreshArgs),
}
